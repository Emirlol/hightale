use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use serde::{
	Deserialize,
	Serialize,
};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct GameProfile {
	pub uuid: Uuid,
	pub username: String,
}

#[derive(Deserialize)]
struct LauncherDataResponse {
	profiles: Vec<GameProfile>,
}

#[derive(Deserialize)]
pub struct GameSessionResponse {
	#[serde(rename = "sessionToken")]
	pub session_token: String,
	#[serde(rename = "identityToken")]
	pub identity_token: String,
	#[serde(rename = "expiresAt")]
	pub expires_at: Option<String>,
}

#[derive(Serialize)]
struct CreateSessionRequest {
	uuid: Uuid,
}

#[derive(Serialize)]
struct AuthGrantRequest<'a> {
	#[serde(rename = "identityToken")]
	identity_token: &'a str,
	aud: &'a str,
}

#[derive(Deserialize)]
struct AuthGrantResponse {
	#[serde(rename = "authorizationGrant")]
	authorization_grant: String,
}

#[derive(Serialize)]
struct ExchangeRequest<'a> {
	#[serde(rename = "authorizationGrant")]
	authorization_grant: &'a str,
	#[serde(rename = "x509Fingerprint")]
	fingerprint: &'a str,
}

#[derive(Deserialize)]
struct AccessTokenResponse {
	#[serde(rename = "accessToken")]
	access_token: String,
}

#[derive(Deserialize)]
pub struct OAuthTokenResponse {
	pub(crate) access_token: String,
	pub(crate) refresh_token: Option<String>,
	pub(crate) id_token: Option<String>,
	pub(crate) expires_in: i64,
	// error fields might be present in error cases, handled by reqwest error checking usually
}

pub struct SessionService {
	client: Client,
	session_url: String,
	account_url: String,
}

impl SessionService {
	pub fn new() -> Result<Self> {
		Ok(Self {
			client: Client::builder().timeout(Duration::from_secs(5)).user_agent("HytaleServer/0.1.0").build()?,
			session_url: "https://sessions.hytale.com".to_string(),
			account_url: "https://account-data.hytale.com".to_string(),
		})
	}

	pub async fn get_game_profiles(&self, oauth_access_token: &str) -> Result<Vec<GameProfile>> {
		let url = format!("{}/my-account/get-profiles", self.account_url);

		let resp = self.client.get(&url)
			.bearer_auth(oauth_access_token)
			.send()
			.await?;

		let data: LauncherDataResponse = resp.error_for_status()?.json().await?;
		Ok(data.profiles)
	}

	pub async fn create_game_session(&self, oauth_access_token: &str, profile_uuid: Uuid) -> Result<GameSessionResponse> {
		let url = format!("{}/game-session/new", self.session_url);

		let body = CreateSessionRequest { uuid: profile_uuid };

		let resp = self.client.post(&url)
			.bearer_auth(oauth_access_token)
			.json(&body)
			.send()
			.await?;

		let data: GameSessionResponse = resp.error_for_status()?.json().await?;
		Ok(data)
	}
	
	pub async fn request_auth_grant(
		&self,
		player_identity_token: &str,
		server_session_token: &str,
		server_id: &str, // The "Audience"
	) -> Result<String> {
		let url = format!("{}/server-join/auth-grant", self.session_url);

		let body = AuthGrantRequest {
			identity_token: player_identity_token,
			aud: server_id,
		};

		let resp = self.client.post(&url).bearer_auth(server_session_token).json(&body).send().await?;

		let data: AuthGrantResponse = resp.error_for_status()?.json().await?;
		Ok(data.authorization_grant)
	}

	pub async fn exchange_grant(&self, grant: &str, cert_fingerprint: &str, server_session_token: &str) -> Result<String> {
		let url = format!("{}/server-join/auth-token", self.session_url);

		let body = ExchangeRequest {
			authorization_grant: grant,
			fingerprint: cert_fingerprint,
		};

		let resp = self.client.post(&url).bearer_auth(server_session_token).json(&body).send().await?;

		let data: AccessTokenResponse = resp.error_for_status()?.json().await?;
		Ok(data.access_token)
	}

	pub async fn refresh_session(&self, current_token: &str) -> Result<GameSessionResponse> {
		let url = format!("{}/game-session/refresh", self.session_url);

		let resp = self.client.post(&url)
			.bearer_auth(current_token)
			.send()
			.await?;

		let data: GameSessionResponse = resp.error_for_status()?.json().await?;
		Ok(data)
	}
}
