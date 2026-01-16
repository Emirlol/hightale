use anyhow::Context;
use base64::{
	engine::general_purpose::URL_SAFE_NO_PAD,
	Engine,
};
use rcgen::CertifiedKey;
use rustls::pki_types::{
	CertificateDer,
	PrivateKeyDer,
	PrivatePkcs8KeyDer,
};
use sha2::{
	Digest,
	Sha256,
};

pub struct ServerCert {
	pub chain: Vec<CertificateDer<'static>>,
	pub key: PrivateKeyDer<'static>,
	pub fingerprint: String,
}

/// Generates a self-signed certificate and private key for QUIC TLS 1.3.
pub fn generate_self_signed_cert() -> anyhow::Result<ServerCert> {
	let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string(), "[::1]".to_string()];

	let CertifiedKey { cert, signing_key } = rcgen::generate_simple_self_signed(subject_alt_names).context("Failed to generate self-signed certificate")?;

	let cert_der = cert.der().to_vec();
	let key_der = signing_key.serialize_der();

	let mut hasher = Sha256::new();
	hasher.update(&cert_der);
	let hash = hasher.finalize();
	let fingerprint = URL_SAFE_NO_PAD.encode(hash);

	let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der));
	let cert_chain = vec![CertificateDer::from(cert_der)];

	Ok(ServerCert { chain: cert_chain, key, fingerprint })
}
