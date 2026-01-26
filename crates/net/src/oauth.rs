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
	Header,
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
					let _ = request.respond(Response::from_string(RESPONSE_HTML).with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).expect("Valid headers")));

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

// language=html // Intellij-based IDEs hint for HTML language injection
const RESPONSE_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
	<meta charset="UTF-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<title>Authentication Successful - Hightale</title>
	<link rel="preconnect" href="https://fonts.googleapis.com">
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
	<link href="https://fonts.googleapis.com/css2?family=Lexend:wght@700&family=Nunito+Sans:wght@400;700&display=swap" rel="stylesheet">
	<style>
		* { margin: 0; padding: 0; box-sizing: border-box; }
		html { color-scheme: dark; background: linear-gradient(180deg, #15243A, #0F1418); min-height: 100vh; }
		body { font-family: "Nunito Sans", sans-serif; color: #b7cedd; min-height: 100vh; display: flex; align-items: center; justify-content: center; padding: 20px; }
		.card { background: rgba(0,0,0,0.4); border: 2px solid rgba(71,81,107,0.6); border-radius: 12px; padding: 48px 40px; max-width: 420px; text-align: center; }
		.icon { width: 64px; height: 64px; margin: 0 auto 24px; border-radius: 50%; display: flex; align-items: center; justify-content: center; }
		.icon svg { width: 32px; height: 32px; }
		.icon-success { background: linear-gradient(135deg, #2d5a3d, #1e3a2a); border: 2px solid #4a9d6b; }
		.icon-success svg { color: #6fcf97; }
		.icon-error { background: linear-gradient(135deg, #5a2d3d, #3a1e2a); border: 2px solid #c3194c; }
		.icon-error svg { color: #ff6b8a; }
		h1 { font-family: "Lexend", sans-serif; font-size: 1.5rem; text-transform: uppercase; background: linear-gradient(#f5fbff, #bfe6ff); -webkit-background-clip: text; background-clip: text; color: transparent; margin-bottom: 12px; }
		p { line-height: 1.6; }
		.error { background: rgba(195,25,76,0.15); border: 1px solid rgba(195,25,76,0.4); border-radius: 6px; padding: 12px; margin-top: 16px; color: #ff8fa8; font-size: 0.875rem; word-break: break-word; }
	</style>
</head>
<body>
	<div class="card">
		<div class="icon icon-success">
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
				<polyline points="20 6 9 17 4 12"/>
			</svg>
		</div>
		<h1>Authentication Successful</h1>
		<p>You have been logged in successfully. You can now close this window and return to the server.</p>
	</div>
</body>
</html>
"#;
