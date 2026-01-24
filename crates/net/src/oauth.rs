use std::{
	collections::HashMap,
	error::Error,
};

use base64::{
	engine::general_purpose::URL_SAFE_NO_PAD,
	Engine,
};
use rand::RngCore;
use reqwest::StatusCode;
use serde::Serialize;
use sha2::{
	Digest,
	Sha256,
};
use thiserror::Error;
use tiny_http::{
	ListenAddr,
	Response,
	Server,
};
use tokio::sync::oneshot;

use crate::{
	api::OAuthTokenResponse,
	oauth::InteractiveLoginError::{
		CallbackServerError,
		GetServerPortError,
		OAuthError,
		RequestError,
	},
};

#[derive(Serialize)]
struct StatePayload {
	state: String,
	port: String,
}

#[derive(Debug, Error)]
pub enum InteractiveLoginError {
	#[error("Failed to start callback server: {0}")]
	CallbackServerError(#[from] Box<dyn Error + Send + Sync>),
	#[error("Failed to get server port: {0}")]
	GetServerPortError(&'static str),
	#[error("Failed to serialize state payload: {0}")]
	SerializeStateError(#[from] serde_json::Error),
	#[error("OAuth error: {0}: {1}")]
	OAuthError(StatusCode, String),
	#[error("Request error: {0}")]
	RequestError(#[from] reqwest::Error),
}

pub struct PendingOAuthState {
	pub code_verifier: String,
	pub redirect_uri: String,
}

/// Starts the OAuth flow.
/// Returns the URL the user needs to visit.
/// Starts a background thread to listen for the callback.
pub fn start_listener() -> Result<(String, PendingOAuthState, oneshot::Receiver<String>), InteractiveLoginError> {
	let (state, code_verifier, code_challenge) = pkce_setup();

	let server = Server::http("127.0.0.1:0").map_err(CallbackServerError)?;

	let port = match server.server_addr() {
		ListenAddr::IP(a) => a.port(),
		ListenAddr::Unix(_) => return Err(GetServerPortError("Callback server is listening on a Unix socket, not TCP")), // Should not happen, but just in case
	};

	const REDIRECT_URI: &str = "https://accounts.hytale.com/consent/client";

	let state_payload = StatePayload {
		state: state.clone(),
		port: port.to_string(),
	};
	let state_json = serde_json::to_string(&state_payload)?;
	let encoded_state = URL_SAFE_NO_PAD.encode(state_json);

	let auth_url = format!(
		"https://oauth.accounts.hytale.com/oauth2/auth?response_type=code&client_id=hytale-server&redirect_uri={}&state={}&scope=openid+offline+auth:server&code_challenge={}&code_challenge_method=S256",
		urlencoding::encode(REDIRECT_URI),
		encoded_state,
		code_challenge
	);

	let (tx, rx) = oneshot::channel::<String>();
	let expected_raw_state = state.clone();

	tokio::task::spawn_blocking(move || {
		if let Ok(request) = server.recv() {
			let url = request.url().to_string();

			if let Ok(parsed_url) = url::Url::parse(&format!("http://localhost{}", url)) {
				let pairs: HashMap<_, _> = parsed_url.query_pairs().collect();

				let code = pairs.get("code").map(|c| c.to_string());
				let ret_state = pairs.get("state").map(|s| s.to_string());

				if let Some(code) = code
					&& let Some(ret_state) = ret_state
					&& ret_state == expected_raw_state
				{
					let _ = request.respond(
						Response::from_string("<html><body><h1 style='color:green'>Authentication Successful!</h1><p>You can return to the console.</p><script>window.close()</script></body></html>")
							.with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap()),
					);

					let _ = tx.send(code);
					return;
				}
			}

			let _ = request.respond(Response::from_string("Login Failed or Invalid State."));
		}
	});

	Ok((
		auth_url,
		PendingOAuthState {
			code_verifier,
			redirect_uri: REDIRECT_URI.to_string(),
		},
		rx,
	))
}

fn pkce_setup() -> (String, String, String) {
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

	(state, code_verifier, code_challenge)
}

pub async fn exchange_code(code: &str, pending_oauth_state: &PendingOAuthState) -> Result<OAuthTokenResponse, InteractiveLoginError> {
	let params = [
		("grant_type", "authorization_code"),
		("client_id", "hytale-server"),
		("code", code),
		("redirect_uri", &pending_oauth_state.redirect_uri),
		("code_verifier", &pending_oauth_state.code_verifier),
	];

	let client = reqwest::Client::new();

	let response = client.post("https://oauth.accounts.hytale.com/oauth2/token").form(&params).send().await.map_err(RequestError)?;

	if !response.status().is_success() {
		let status = response.status();
		let text = response.text().await.unwrap_or_default();
		return Err(OAuthError(status, text));
	}

	let token_data: OAuthTokenResponse = response.json().await.map_err(RequestError)?;
	Ok(token_data)
}
