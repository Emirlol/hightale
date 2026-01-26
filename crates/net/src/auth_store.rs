use std::path::PathBuf;

use aes_gcm::{
	aead::{
		rand_core::RngCore,
		Aead,
		KeyInit,
		OsRng,
	},
	Aes256Gcm,
	Nonce,
};
use anyhow::{
	Context,
	Result,
};
use chrono::{
	DateTime,
	Utc,
};
use pbkdf2::pbkdf2_hmac;
use serde::{
	Deserialize,
	Serialize,
};
use sha2::Sha256;
use tracing::warn;
use uuid::Uuid;

const SALT: &[u8] = b"HytaleAuthCredentialStore";
const PBKDF2_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;
const IV_LEN: usize = 12;

type EncryptionKey = [u8; KEY_LEN];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAuthTokens {
	pub session_token: String,
	pub identity_token: String,
	pub refresh_token: Option<String>,
	pub expires_at: Option<DateTime<Utc>>,
	pub server_id: Option<Uuid>,
}

pub struct AuthFileStore {
	path: PathBuf,
	key: Option<EncryptionKey>,
}

impl AuthFileStore {
	pub fn new(path: PathBuf) -> Self {
		let key = derive_key().ok().flatten();
		if key.is_none() {
			warn!("Cannot derive encryption key - auth.enc persistence disabled");
		}
		Self { path, key }
	}

	pub fn load(&self) -> Result<Option<StoredAuthTokens>> {
		if self.key.is_none() {
			return Ok(None);
		}
		if !self.path.exists() {
			return Ok(None);
		}

		let encrypted = std::fs::read(&self.path).with_context(|| format!("Reading auth store: {}", self.path.display()))?;
		let Some(plaintext) = self.decrypt(&encrypted) else {
			warn!("Failed to decrypt auth store {}", self.path.display());
			return Ok(None);
		};

		let tokens = serde_json::from_slice::<StoredAuthTokens>(&plaintext).with_context(|| format!("Parsing auth store: {}", self.path.display()))?;
		Ok(Some(tokens))
	}

	pub fn save(&self, tokens: &StoredAuthTokens) -> Result<()> {
		let Some(_) = self.key else {
			return Ok(());
		};
		let plaintext = serde_json::to_vec(tokens).context("Serializing auth store")?;
		let Some(encrypted) = self.encrypt(&plaintext) else {
			return Ok(());
		};
		std::fs::write(&self.path, encrypted).with_context(|| format!("Writing auth store: {}", self.path.display()))?;
		Ok(())
	}

	pub fn clear(&self) -> Result<()> {
		if self.path.exists() {
			std::fs::remove_file(&self.path).with_context(|| format!("Remmoving auth store: {}", self.path.display()))?;
		}
		Ok(())
	}

	fn encrypt(&self, plaintext: &[u8]) -> Option<Vec<u8>> {
		let key = self.key.as_ref()?;
		let cipher = Aes256Gcm::new_from_slice(key).ok()?;
		let mut iv = [0u8; IV_LEN];
		let mut rng = OsRng;
		rng.fill_bytes(&mut iv);
		let nonce = Nonce::from_slice(&iv);
		let ciphertext = cipher.encrypt(nonce, plaintext).ok()?;

		let mut out = Vec::with_capacity(IV_LEN + ciphertext.len());
		out.extend_from_slice(&iv);
		out.extend_from_slice(&ciphertext);
		Some(out)
	}

	fn decrypt(&self, encrypted: &[u8]) -> Option<Vec<u8>> {
		let key = self.key.as_ref()?;
		if encrypted.len() < IV_LEN {
			return None;
		}
		let cipher = Aes256Gcm::new_from_slice(key).ok()?;
		let (iv, ciphertext) = encrypted.split_at(IV_LEN);
		let nonce = Nonce::from_slice(iv);
		cipher.decrypt(nonce, ciphertext).ok()
	}
}

fn derive_key() -> Result<Option<EncryptionKey>> {
	let machine_id = match machine_uid::get() {
		Ok(id) if !id.trim().is_empty() => id,
		_ => return Ok(None),
	};

	let mut key = [0u8; KEY_LEN];
	pbkdf2_hmac::<Sha256>(machine_id.as_bytes(), SALT, PBKDF2_ITERATIONS, &mut key);
	Ok(Some(key))
}
