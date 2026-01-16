use std::{
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
use rand::{
	Rng,
	RngCore,
};
use reqwest::Client;
use serde::{
	Deserialize,
	Serialize,
};
use sha2::{
	Digest,
	Sha256,
};
use tiny_http::{
	ListenAddr,
	Response,
	Server,
};
use tracing::{
	error,
	info,
	log::warn,
};
use uuid::Uuid;

use crate::api::SessionService;

#[derive(Debug, Clone)]
pub struct ServerSession {
	pub session_token: String,
	pub identity_token: String,
	pub refresh_token: Option<String>,
	pub server_id: Uuid,
	pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct OAuthTokenResponse {
	access_token: String,
	refresh_token: Option<String>,
	id_token: Option<String>,
	expires_in: i64,
	// error fields might be present in error cases, handled by reqwest error checking usually
}

#[derive(Deserialize)]
struct JwtClaims {
	exp: Option<i64>,
}

#[derive(Serialize)]
struct StatePayload {
	state: String,
	port: String,
}

pub struct ServerAuthManager {
	api: SessionService,
	session: Arc<RwLock<Option<ServerSession>>>,
	cert_fingerprint: String,
	http_client: Client,
	instance_audience_id: String,
}

impl ServerAuthManager {
	pub fn new(cert_fingerprint: String) -> Result<Arc<Self>> {
		Ok(Arc::new(Self {
			api: SessionService::new()?,
			session: Arc::new(RwLock::new(None)),
			cert_fingerprint,
			http_client: Client::builder().user_agent("HytaleServer/1.0.0").timeout(std::time::Duration::from_secs(10)).build()?,
			instance_audience_id: Uuid::new_v4().to_string(),
		}))
	}

	/// Entry point: Tries to load tokens from Env and starts the background loop.
	pub async fn initialize(self: &Arc<Self>) -> Result<()> {
		info!("Initializing ServerAuthManager...");

		let session_token = std::env::var("HYTALE_SERVER_SESSION_TOKEN").ok();
		let identity_token = std::env::var("HYTALE_SERVER_IDENTITY_TOKEN").ok();

		if let (Some(sess), Some(id)) = (session_token, identity_token) {
			info!("Tokens found in environment variables.");

			let expires_at = parse_jwt_expiry(&sess).context("Failed to parse session token expiry")?;

			let new_session = ServerSession {
				session_token: sess,
				identity_token: id,
				refresh_token: None,
				server_id: Uuid::new_v4(),
				expires_at,
			};

			{
				let mut lock = self.session.write();
				*lock = Some(new_session);
			}

			self.clone().spawn_refresh_loop();

			info!("Auth initialized successfully.");
		} else {
			warn!("No tokens found in environment. Server starts unauthenticated.");
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

	/// The Background Task (Replaces ScheduledExecutorService)
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

	/// Logic to determine wait time (Java: refreshDelay logic)
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

		info!("Session token refreshed successfully.");
		Ok(())
	}

	/// Starts the OAuth flow.
	/// Returns the URL the user needs to visit.
	/// Starts a background thread to listen for the callback.
	pub async fn start_browser_flow(self: &Arc<Self>) -> Result<String> {
		let mut rng = rand::rng();
		let mut state_buf = [0u8; 16]; // 32 chars
		rng.fill_bytes(&mut state_buf);
		let state: String = hex::encode(state_buf);

		let mut code_verifier_buf = [0u8; 32]; // 64 chars
		rng.fill_bytes(&mut code_verifier_buf);
		let code_verifier: String = hex::encode(code_verifier_buf);

		let mut hasher = Sha256::new();
		hasher.update(code_verifier.as_bytes());
		let challenge_bytes = hasher.finalize();
		let code_challenge = URL_SAFE_NO_PAD.encode(challenge_bytes);

		let server = Server::http("127.0.0.1:0").map_err(|e| anyhow::anyhow!("Failed to start callback server: {}", e))?;

		let port = match server.server_addr() {
			ListenAddr::IP(a) => a.port(),
			#[cfg(unix)]
			ListenAddr::Unix(_) => return Err(anyhow::anyhow!("Callback server is listening on a Unix socket, not TCP")), // Should not happen, but just in case
		};

		let redirect_uri = "https://accounts.hytale.com/consent/client";

		let state_payload = StatePayload {
			state: state.clone(),
			port: port.to_string(),
		};
		let state_json = serde_json::to_string(&state_payload)?;
		let encoded_state = URL_SAFE_NO_PAD.encode(state_json);

		let auth_url = format!(
			"https://oauth.accounts.hytale.com/oauth2/auth?response_type=code&client_id=hytale-server&redirect_uri={}&state={}&scope=openid+offline+auth:server&code_challenge={}&code_challenge_method=S256",
			urlencoding::encode(redirect_uri),
			encoded_state,
			code_challenge
		);

		let auth_manager = self.clone();
		let expected_raw_state = state.clone();

		tokio::task::spawn_blocking(move || {
			if let Ok(request) = server.recv() {
				let url = request.url().to_string();

				if let Ok(parsed_url) = url::Url::parse(&format!("http://localhost{}", url)) {
					let pairs: std::collections::HashMap<_, _> = parsed_url.query_pairs().collect();

					let code = pairs.get("code").map(|c| c.to_string());
					let ret_state = pairs.get("state").map(|s| s.to_string());

					if let Some(code) = code
						&& let Some(ret_state) = ret_state
						&& ret_state == expected_raw_state
					{
						// We're in a blocking thread, so we need to enter the async runtime
						let rt_handle = tokio::runtime::Handle::current();
						let exchange_result = rt_handle.block_on(async { auth_manager.exchange_code(&code, &code_verifier, redirect_uri).await });

						match exchange_result {
							Ok(_) => {
								let _ = request.respond(
									Response::from_string(
										"<html><body><h1 style='color:green'>Authentication Successful!</h1><p>You can return to the console.</p><script>window.close()</script></body></html>",
									)
									.with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap()),
								);
							}
							Err(e) => {
								error!("Token exchange failed: {}", e);
								let _ = request.respond(Response::from_string(format!("Token Exchange Failed: {}", e)));
							}
						}
						return;
					}
				}

				let _ = request.respond(Response::from_string("Login Failed or Invalid State."));
			}
		});

		Ok(auth_url)
	}

	pub async fn exchange_code(&self, code: &str, code_verifier: &str, redirect_uri: &str) -> Result<()> {
		info!("Exchanging authorization code for tokens...");

		let params = [
			("grant_type", "authorization_code"),
			("client_id", "hytale-server"),
			("code", code),
			("redirect_uri", redirect_uri),
			("code_verifier", code_verifier),
		];

		let response = self.http_client.post("https://oauth.accounts.hytale.com/oauth2/token").form(&params).send().await?;

		if !response.status().is_success() {
			let status = response.status();
			let text = response.text().await.unwrap_or_default();
			return Err(anyhow!("OAuth Error {}: {}", status, text));
		}

		let token_data: OAuthTokenResponse = response.json().await.context("Failed to parse OAuth response")?;

		info!("Fetching Game Profiles...");
		let profiles = self.api.get_game_profiles(&token_data.access_token).await?;

		// Auto-select first profile (Java logic does this if --owner-uuid not set)
		let profile = profiles.first().ok_or(anyhow!("No game profiles found on this account"))?;
		info!("Selected Profile: {} ({})", profile.username, profile.uuid);

		// Extract Identity to get Server ID (Audience)
		let id_token = token_data.id_token.ok_or(anyhow!("Missing id_token in response"))?;

		// Parse the JWT to get the 'sub' (Server UUID)
		// We use a simplified parse here (assuming signature validation isn't strictly required for self-obtained token,
		// OR reuse the validation logic from previous steps if strictness is needed).
		let server_id = parse_sub_from_jwt(&id_token)?;

		info!("Creating Game Session...");
		let game_session = self.api.create_game_session(&token_data.access_token, profile.uuid).await
			.context("Failed to create game session")?;

		let session = ServerSession {
			session_token: game_session.session_token,
			identity_token: game_session.identity_token,
			refresh_token: token_data.refresh_token,
			server_id,
			expires_at: Some(Utc::now() + chrono::Duration::seconds(token_data.expires_in)),
		};

		{
			let mut lock = self.session.write();
			*lock = Some(session);
		}

		info!("Successfully authenticated as server ID: {}", server_id);
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
fn parse_sub_from_jwt(token: &str) -> Result<Uuid> {
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
