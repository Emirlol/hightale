pub mod assets;
pub mod commands;
pub mod console;
pub mod options;

use std::sync::{
	Arc,
	RwLock,
};

use command::CommandRegistry;
use is_terminal::IsTerminal;
use net::{
	auth::ServerAuthManager,
	server::QuicServer,
	tls,
};
use tokio::sync::mpsc;
use tracing::{
	error,
	info,
};
use tracing_subscriber::{
	layer::SubscriberExt,
	util::SubscriberInitExt,
	EnvFilter,
};

macro_rules! register_commands {
    ($registry_lock:expr, $( $register:path => ( $( $arg:expr ),* $(,)? ) ),+ $(,)? ) => {{
        let mut registry = $registry_lock.write().unwrap();
        $(
            $register(&mut registry, $( $arg ),*);
        )+
    }};
}

pub async fn main() -> anyhow::Result<()> {
	let options = options::ServerOptions::load()?;

	let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
	let command_registry = CommandRegistry::new();
	let cmd_reg_wrap = Arc::new(RwLock::new(command_registry));
	if std::env::var("RUST_LOG").is_err() {
		// Make sure we have the default log level
		unsafe {
			std::env::set_var("RUST_LOG", "info");
		}
	}
	if std::io::stdout().is_terminal() {
		let (writer, console_task) = console::setup_interactive(cmd_reg_wrap.clone(), shutdown_tx.clone())?;
		tracing_subscriber::registry()
			.with(tracing_subscriber::fmt::layer().with_writer(move || writer.clone()))
			.with(EnvFilter::from_default_env())
			.init();

		std::thread::spawn(console_task);
	} else {
		// Headless mode
		tracing_subscriber::registry().with(tracing_subscriber::fmt::layer()).with(EnvFilter::from_default_env()).init();
	};

	rustls::crypto::ring::default_provider().install_default().unwrap();
	let cert_data = tls::generate_self_signed_cert()?;
	let fingerprint = cert_data.fingerprint.clone();
	info!("Generated self-signed cert. Fingerprint: {}", fingerprint);

	let auth_manager = ServerAuthManager::new(fingerprint, Some(options.auth_store_path.clone()))?;
	auth_manager.initialize(options.auth_session_token.clone(), options.auth_identity_token.clone()).await?;
	let rt = tokio::runtime::Handle::current();

	register_commands!(cmd_reg_wrap,
		commands::auth::register => (auth_manager.clone(), rt.clone()),
		commands::help::register => (),
		commands::stop::register => (shutdown_tx),
	);

	let common_assets = Arc::new(assets::load_common_assets(&options.assets_dir)?);

	let quic_options = net::server::QuicServerOptions {
		max_idle_timeout: std::time::Duration::from_secs(options.quic_idle_timeout_secs),
		keep_alive_interval: std::time::Duration::from_secs(options.quic_keep_alive_secs),
	};
	let server = QuicServer::bind(options.bind_addr, cert_data, auth_manager, common_assets, quic_options).await?;

	info!("Server is Ready.");

	tokio::select! {
		// The QUIC accept loop
		_ = server.run_accept_loop() => {
			error!("Server network loop exited unexpectedly");
		}
		// Graceful shutdown
		_ = shutdown_rx.recv() => {
			info!("Shutdown signal received.");
			server.close();
		}
	}

	Ok(())
}
