use std::{
	fmt::Debug,
	net::SocketAddr,
	sync::Arc,
	time::Duration,
};

use anyhow::{
	Context,
	Result,
};
use quinn::{
	crypto::rustls::QuicServerConfig,
	Endpoint,
	ServerConfig,
	TransportConfig,
};
use rustls::{
	client::danger::HandshakeSignatureValid,
	crypto::WebPkiSupportedAlgorithms,
	pki_types::CertificateDer,
	server::danger::{
		ClientCertVerified,
		ClientCertVerifier,
	},
	DigitallySignedStruct,
	DistinguishedName,
	Error,
	SignatureScheme,
};
use tracing::{
	error,
	info,
};

use crate::{
	auth::ServerAuthManager,
	connection::PlayerConnection,
	tls::ServerCert,
};

/// A verifier that requests a client certificate but trusts ANYTHING.
/// This matches Java's `InsecureTrustManagerFactory.INSTANCE` behavior.
#[derive(Debug)]
struct AllowAnyClientCertVerifier {
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

	fn verify_client_cert(&self, _end_entity: &CertificateDer<'_>, _intermediates: &[CertificateDer<'_>], _now: rustls::pki_types::UnixTime) -> Result<ClientCertVerified, rustls::Error> {
		// Blindly trust the client certificate.
		// We only need it to extract the public key/fingerprint later for Auth.
		Ok(ClientCertVerified::assertion())
	}

	fn verify_tls12_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> std::result::Result<HandshakeSignatureValid, Error> {
		rustls::crypto::verify_tls12_signature(message, cert, dss, &self.supported_algs)
	}

	fn verify_tls13_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> std::result::Result<HandshakeSignatureValid, Error> {
		rustls::crypto::verify_tls13_signature(message, cert, dss, &self.supported_algs)
	}

	fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
		self.supported_algs.supported_schemes()
	}
}

pub struct QuicServer {
	endpoint: Endpoint,
	auth_manager: Arc<ServerAuthManager>,
}

impl QuicServer {
	pub async fn bind(addr: SocketAddr, cert: ServerCert, auth_manager: Arc<ServerAuthManager>) -> Result<Self> {
		info!("Setting up QUIC transport...");

		let ServerCert { chain, key, fingerprint } = cert;

		let mut tls = rustls::ServerConfig::builder()
			.with_client_cert_verifier(Arc::new(AllowAnyClientCertVerifier::new()))
			.with_single_cert(chain, key)
			.context("Failed to create TLS config")?;

		// Make sure only clients speaking "hytale" can connect.
		tls.alpn_protocols = vec![b"hytale/1".to_vec()];

		let mut server_config = ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(tls)?));

		let mut transport_config = TransportConfig::default();
		// Disconnect clients if they are silent for 30 seconds
		transport_config.max_idle_timeout(Some(Duration::from_secs(30).try_into()?));
		transport_config.keep_alive_interval(Some(Duration::from_secs(5)));

		server_config.transport_config(Arc::new(transport_config));

		let endpoint = Endpoint::server(server_config, addr)?;

		info!("QUIC Listener bound to {}", endpoint.local_addr()?);

		Ok(Self { endpoint, auth_manager })
	}

	/// The main accept loop. Blocks until the server shuts down.
	pub async fn run_accept_loop(&self) {
		info!("Ready to accept connections.");

		while let Some(connecting) = self.endpoint.accept().await {
			let auth = self.auth_manager.clone();

			tokio::spawn(async move {
				if let Err(e) = handle_connection(connecting, auth).await {
					error!("Connection terminated with error: {}", e);
				}
			});
		}

		info!("QUIC Endpoint shutdown.");
	}

	/// Graceful shutdown
	pub fn close(self) {
		self.endpoint.close(0u32.into(), b"Server shutting down");
	}
}

/// Handles the lifecycle of a single player connection
async fn handle_connection(connecting: quinn::Incoming, auth: Arc<ServerAuthManager>) -> Result<()> {
	let connection = connecting.await?;
	let remote_addr = connection.remote_address();

	info!("New connection from {}", remote_addr);

	let (send_stream, recv_stream) = connection.accept_bi().await.context("Failed to open bidirectional stream")?;

	let player_conn = PlayerConnection::new(connection, send_stream, recv_stream, auth);

	player_conn.run().await?;

	info!("{} disconnected cleanly.", remote_addr);

	Ok(())
}
