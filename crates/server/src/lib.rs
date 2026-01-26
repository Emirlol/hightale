pub mod assets;
pub mod commands;
pub mod console;
pub mod options;

use std::{
	str::FromStr,
	sync::{
		Arc,
		RwLock,
	},
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
	Level,
};
use tracing_subscriber::{
	filter::Directive,
	layer::SubscriberExt,
	util::SubscriberInitExt,
	EnvFilter,
	Layer,
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
	eprintln!("RUST_LOG={:?}", std::env::var("RUST_LOG"));
	if std::io::stdout().is_terminal() {
		let (writer, console_task) = console::setup_interactive(cmd_reg_wrap.clone(), shutdown_tx.clone())?;
		let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|err| {
			eprintln!("Invalid RUST_LOG: {}", err);
			EnvFilter::new("info")
		});
		let console_filter = EnvFilter::new("server::console=info");
		let console_layer = tracing_subscriber::fmt::layer().with_writer(move || writer.clone()).with_filter(console_filter);
		let stderr_filter = env_filter.add_directive(Directive::from_str("server::console=off")?);
		let stderr_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stderr).with_filter(stderr_filter);
		tracing_subscriber::registry().with(console_layer).with(stderr_layer).init();

		std::thread::spawn(console_task);
	} else {
		// Headless mode
		let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|err| {
			eprintln!("Invalid RUST_LOG: {}", err);
			EnvFilter::new("info")
		});
		tracing_subscriber::registry().with(tracing_subscriber::fmt::layer()).with(env_filter).init();
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
