use anyhow::Context;
use base64::{
	engine::general_purpose::URL_SAFE_NO_PAD,
	Engine,
};
use rcgen::CertifiedKey;
use rustls::{
	client::danger::HandshakeSignatureValid,
	crypto::WebPkiSupportedAlgorithms,
	pki_types::{
		CertificateDer,
		PrivateKeyDer,
		PrivatePkcs8KeyDer,
	},
	server::danger::{
		ClientCertVerified,
		ClientCertVerifier,
	},
	DigitallySignedStruct,
	DistinguishedName,
	Error,
	SignatureScheme,
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

/// A verifier that requests a client certificate but trusts ANYTHING.
/// This matches Java's `InsecureTrustManagerFactory.INSTANCE` behavior.
#[derive(Debug)]
pub struct AllowAnyClientCertVerifier {
	supported_algs: WebPkiSupportedAlgorithms,
}

impl AllowAnyClientCertVerifier {
	fn new() -> Self {
		Self {
			supported_algs: rustls::crypto::CryptoProvider::get_default()
				.expect("No default crypto provider found")
				.signature_verification_algorithms,
		}
	}
}

impl Default for AllowAnyClientCertVerifier {
	fn default() -> Self {
		Self::new()
	}
}

impl ClientCertVerifier for AllowAnyClientCertVerifier {
	fn offer_client_auth(&self) -> bool {
		true
	}

	fn client_auth_mandatory(&self) -> bool {
		true
	}

	fn root_hint_subjects(&self) -> &[DistinguishedName] {
		&[]
	}

	fn verify_client_cert(&self, _end_entity: &CertificateDer<'_>, _intermediates: &[CertificateDer<'_>], _now: rustls::pki_types::UnixTime) -> anyhow::Result<ClientCertVerified, Error> {
		// Blindly trust the client certificate.
		// We only need it to extract the public key/fingerprint later for Auth.
		Ok(ClientCertVerified::assertion())
	}

	fn verify_tls12_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> {
		rustls::crypto::verify_tls12_signature(message, cert, dss, &self.supported_algs)
	}

	fn verify_tls13_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> {
		rustls::crypto::verify_tls13_signature(message, cert, dss, &self.supported_algs)
	}

	fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
		self.supported_algs.supported_schemes()
	}
}
