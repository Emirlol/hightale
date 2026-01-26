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
use common_assets::CommonAssetStore;
use quinn::{
	crypto::rustls::QuicServerConfig,
	Endpoint,
	ServerConfig,
	TransportConfig,
};
use tracing::{
	error,
	info,
};

use crate::{
	auth::ServerAuthManager,
	connection::PlayerConnection,
	tls::{
		AllowAnyClientCertVerifier,
		ServerCert,
	},
};

pub struct QuicServer {
	endpoint: Endpoint,
	auth_manager: Arc<ServerAuthManager>,
	common_assets: Arc<CommonAssetStore>,
}

#[derive(Clone, Debug)]
pub struct QuicServerOptions {
	pub max_idle_timeout: Duration,
	pub keep_alive_interval: Duration,
}

impl Default for QuicServerOptions {
	fn default() -> Self {
		Self {
			max_idle_timeout: Duration::from_secs(30),
			keep_alive_interval: Duration::from_secs(5),
		}
	}
}

impl QuicServer {
	pub async fn bind(addr: SocketAddr, cert: ServerCert, auth_manager: Arc<ServerAuthManager>, common_assets: Arc<CommonAssetStore>, options: QuicServerOptions) -> Result<Self> {
		info!("Setting up QUIC transport...");

		let ServerCert { chain, key, fingerprint: _ } = cert;

		let mut tls = rustls::ServerConfig::builder()
			.with_client_cert_verifier(Arc::new(AllowAnyClientCertVerifier::default()))
			.with_single_cert(chain, key)
			.context("Failed to create TLS config")?;

		// Make sure only clients speaking "hytale" can connect.
		tls.alpn_protocols = vec![b"hytale/2".to_vec(), b"hytale/1".to_vec()];

		let mut server_config = ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(tls)?));

		let mut transport_config = TransportConfig::default();
		// Disconnect clients if they are silent for 30 seconds
		transport_config.max_idle_timeout(Some(options.max_idle_timeout.try_into()?));
		transport_config.keep_alive_interval(Some(options.keep_alive_interval));

		server_config.transport_config(Arc::new(transport_config));

		let endpoint = Endpoint::server(server_config, addr)?;

		info!("QUIC Listener bound to {}", endpoint.local_addr()?);

		Ok(Self {
			endpoint,
			auth_manager,
			common_assets,
		})
	}

	/// The main accept loop. Blocks until the server shuts down.
	pub async fn run_accept_loop(&self) {
		info!("Ready to accept connections.");

		while let Some(connecting) = self.endpoint.accept().await {
			let auth = self.auth_manager.clone();
			let common_assets = self.common_assets.clone();

			tokio::spawn(async move {
				if let Err(e) = handle_connection(connecting, auth, common_assets).await {
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
async fn handle_connection(connecting: quinn::Incoming, auth: Arc<ServerAuthManager>, common_assets: Arc<CommonAssetStore>) -> Result<()> {
	let connection = connecting.await?;
	let remote_addr = connection.remote_address();

	info!("New connection from {}", remote_addr);

	let (send_stream, recv_stream) = connection.accept_bi().await.context("Failed to open bidirectional stream")?;

	let player_conn = PlayerConnection::new(connection, send_stream, recv_stream, auth, common_assets);

	player_conn.run().await?;

	info!("{} disconnected cleanly.", remote_addr);

	Ok(())
}
