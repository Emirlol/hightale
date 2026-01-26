use std::{
	path::PathBuf,
	sync::Arc,
	time::Duration,
};

use anyhow::{
	anyhow,
	bail,
	Context,
	Result,
};
use base64::{
	engine::general_purpose::URL_SAFE_NO_PAD,
	Engine,
};
use chrono::{
	DateTime,
	Utc,
};
use parking_lot::RwLock;
use serde::Deserialize;
use tracing::{
	error,
	info,
	log::warn,
};
use uuid::Uuid;

use crate::{
	api::{
		OAuthTokenResponse,
		SessionService,
	},
	auth_store::{
		AuthFileStore,
		StoredAuthTokens,
	},
	oauth,
};

#[derive(Debug, Clone)]
pub struct ServerSession {
	pub session_token: String,
	pub identity_token: String,
	pub refresh_token: Option<String>,
	pub server_id: Uuid,
	pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct JwtClaims {
	exp: Option<i64>,
}

pub struct ServerAuthManager {
	api: SessionService,
	session: Arc<RwLock<Option<ServerSession>>>,
	cert_fingerprint: String,
	instance_audience_id: String,
	auth_store: Option<AuthFileStore>,
}

impl ServerAuthManager {
	pub fn new(cert_fingerprint: String, auth_store_path: Option<PathBuf>) -> Result<Arc<Self>> {
		let auth_store = auth_store_path.map(AuthFileStore::new);
		Ok(Arc::new(Self {
			api: SessionService::new()?,
			session: Arc::new(RwLock::new(None)),
			cert_fingerprint,
			instance_audience_id: Uuid::new_v4().to_string(),
			auth_store,
		}))
	}

	/// Entry point: Tries to load tokens from inputs and starts the background loop.
	pub async fn initialize(self: &Arc<Self>, session_token: Option<String>, identity_token: Option<String>) -> Result<()> {
		info!("Initializing ServerAuthManager...");

		let mut session_token = session_token;
		let mut identity_token = identity_token;
		let mut refresh_token = None;
		let mut expires_at = None;
		let mut server_id = None;

		if session_token.is_none()
			&& identity_token.is_none()
			&& let Some(store) = &self.auth_store
			&& let Ok(Some(tokens)) = store.load()
		{
			info!("Loaded auth tokens from encrypted store.");
			session_token = Some(tokens.session_token);
			identity_token = Some(tokens.identity_token);
			refresh_token = tokens.refresh_token;
			expires_at = tokens.expires_at;
			server_id = tokens.server_id;
		}

		if let (Some(sess), Some(id)) = (session_token, identity_token) {
			let expires_at = match expires_at {
				Some(v) => Some(v),
				None => parse_jwt_expiry(&sess).context("Failed to parse session token expiry")?,
			};

			let new_session = ServerSession {
				session_token: sess,
				identity_token: id,
				refresh_token,
				server_id: server_id.unwrap_or_else(Uuid::new_v4),
				expires_at,
			};

			{
				let mut lock = self.session.write();
				*lock = Some(new_session);
			}

			self.persist_session().await;
			self.clone().spawn_refresh_loop();

			info!("Auth initialized successfully.");
		} else {
			warn!("No tokens found. Server starts unauthenticated.");
		}

		Ok(())
	}

	pub async fn validate_player_join(&self, player_identity_token: &str) -> Result<String> {
		let (server_token, server_id) = {
			let lock = self.session.read();
			let s = lock.as_ref().ok_or(anyhow::anyhow!("Server not authenticated"))?;
			(s.session_token.clone(), s.server_id.to_string())
		};

		let grant = self.api.request_auth_grant(player_identity_token, &server_token, &server_id).await?;

		let access_token = self.api.exchange_grant(&grant, &self.cert_fingerprint, &server_token).await?;

		Ok(access_token)
	}

	/// Accessor to check if we are authenticated
	pub async fn is_authenticated(&self) -> bool {
		self.session.read().is_some()
	}

	/// Spawns a background task to refresh the session token when its time is due
	fn spawn_refresh_loop(self: Arc<Self>) {
		tokio::spawn(async move {
			loop {
				let sleep_duration = self.calculate_next_refresh().await;

				info!("Next token refresh scheduled in {} seconds.", sleep_duration.as_secs());
				tokio::time::sleep(sleep_duration).await;

				if let Err(e) = self.perform_refresh().await {
					error!("Token refresh failed: {}. Retrying in 1 minute.", e);
					tokio::time::sleep(Duration::from_secs(60)).await;
				}
			}
		});
	}

	async fn calculate_next_refresh(&self) -> Duration {
		let lock = self.session.read();
		match &*lock {
			Some(session) => {
				if let Some(expiry) = session.expires_at {
					let now = Utc::now();
					let seconds_until_expiry = (expiry - now).num_seconds();
					// Buffer of 300 seconds (5 mins), minimum wait of 60 seconds
					let delay = std::cmp::max(seconds_until_expiry - 300, 60);
					Duration::from_secs(delay as u64)
				} else {
					Duration::from_secs(3600)
				}
			}
			None => Duration::from_secs(10),
		}
	}

	pub async fn start_browser_flow(self: &Arc<Self>) -> Result<String> {
		let (url, pending_state, rx_code) = oauth::start_listener()?;

		let manager = self.clone();

		tokio::spawn(async move {
			info!("Waiting for browser login...");

			match rx_code.await {
				Ok(code) => {
					info!("Code received. Exchanging for tokens...");

					match oauth::exchange_code(&code, &pending_state).await {
						Ok(tokens) => {
							if let Err(e) = manager.complete_login(tokens).await {
								error!("Failed to complete login session: {}", e);
							} else {
								info!("Browser authentication finished successfully!");
							}
						}
						Err(e) => error!("OAuth exchange failed: {}", e),
					}
				}
				Err(_) => error!("Login listener cancelled or failed."),
			}
		});

		Ok(url)
	}

	async fn complete_login(self: &Arc<Self>, tokens: OAuthTokenResponse) -> Result<()> {
		let profiles = self.api.get_game_profiles(&tokens.access_token).await?;
		let profile = profiles.first().ok_or(anyhow::anyhow!("No game profiles found"))?;

		info!("Selected Profile: {}", profile.username);

		let game_session = self.api.create_game_session(&tokens.access_token, profile.uuid).await?;

		let id_token = tokens.id_token.ok_or(anyhow::anyhow!("Missing ID token"))?;
		let server_id = parse_sub_from_jwt(&id_token)?;

		let session = ServerSession {
			session_token: game_session.session_token,
			identity_token: game_session.identity_token,
			refresh_token: tokens.refresh_token,
			server_id,
			expires_at: Some(Utc::now() + chrono::Duration::seconds(tokens.expires_in)),
		};

		*self.session.write() = Some(session);
		self.persist_session().await;
		// Start the refresh loop since we are now logged in
		self.clone().spawn_refresh_loop();

		Ok(())
	}

	/// Performs the HTTP Refresh call
	async fn perform_refresh(&self) -> Result<()> {
		// Read current token
		let current_token = {
			let lock = self.session.read();
			match &*lock {
				Some(s) => s.session_token.clone(),
				None => return Ok(()),
			}
		};

		info!("Refreshing session token...");

		let refreshed_data = self.api.refresh_session(&current_token).await.context("Failed to refresh session via API")?;

		let new_expiry = parse_jwt_expiry(&refreshed_data.identity_token).ok().flatten();

		{
			let mut lock = self.session.write();
			if let Some(existing) = &mut *lock {
				existing.session_token = refreshed_data.session_token;
				existing.identity_token = refreshed_data.identity_token;
				existing.expires_at = new_expiry;
			}
		}

		self.persist_session().await;
		info!("Session token refreshed successfully.");
		Ok(())
	}

	pub async fn get_session_token(&self) -> Option<String> {
		self.session.read().as_ref().map(|s| s.session_token.clone())
	}

	pub async fn get_identity_token(&self) -> Option<String> {
		self.session.read().as_ref().map(|s| s.identity_token.clone())
	}

	pub async fn get_server_id(&self) -> String {
		self.session
			.read()
			.as_ref()
			.map(|s| s.server_id.to_string())
			// Fallback for offline testing or if config missing
			.unwrap_or_else(|| "hytale-server".to_string())
	}

	pub fn get_cert_fingerprint(&self) -> &str {
		&self.cert_fingerprint
	}

	pub fn get_api(&self) -> &SessionService {
		&self.api
	}

	async fn persist_session(&self) {
		let Some(store) = &self.auth_store else {
			return;
		};
		let session = { self.session.read().clone() };
		let Some(session) = session else {
			return;
		};
		let tokens = StoredAuthTokens {
			session_token: session.session_token,
			identity_token: session.identity_token,
			refresh_token: session.refresh_token,
			expires_at: session.expires_at,
			server_id: Some(session.server_id),
		};
		if let Err(e) = store.save(&tokens) {
			warn!("Failed to persist auth tokens: {}", e);
		}
	}

	pub async fn clear_credentials(&self) -> Result<()> {
		*self.session.write() = None;
		if let Some(store) = &self.auth_store {
			store.clear()?;
		}
		Ok(())
	}
}

/// Helper: Parse JWT Expiry without full validation
fn parse_jwt_expiry(token: &str) -> Result<Option<DateTime<Utc>>> {
	let parts: Vec<&str> = token.split('.').collect();
	if parts.len() != 3 {
		return Err(anyhow::anyhow!("Invalid JWT format"));
	}

	let payload_part = parts[1];
	let decoded = URL_SAFE_NO_PAD.decode(payload_part).context("Failed to base64 decode JWT payload")?;

	let claims: JwtClaims = serde_json::from_slice(&decoded).context("Failed to parse JWT JSON")?;

	if let Some(exp) = claims.exp
		&& let Some(dt) = DateTime::from_timestamp(exp, 0)
	{
		Ok(Some(dt))
	} else {
		Ok(None)
	}
}

/// Helper to extract the Subject UUID from the JWT without full validation
pub(crate) fn parse_sub_from_jwt(token: &str) -> Result<Uuid> {
	let parts: Vec<&str> = token.split('.').collect();
	if parts.len() != 3 {
		bail!("Invalid JWT format");
	}

	let payload_part = parts[1];
	let decoded = URL_SAFE_NO_PAD.decode(payload_part).context("Base64 decode failed")?;

	#[derive(Deserialize)]
	struct MinimalClaims {
		sub: String,
	}

	let claims: MinimalClaims = serde_json::from_slice(&decoded).context("JSON parse failed")?;
	Uuid::parse_str(&claims.sub).map_err(|e| anyhow!("Invalid UUID in sub claim: {}", e))
}
