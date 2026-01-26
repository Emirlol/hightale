use std::{
	net::SocketAddr,
	path::{
		Path,
		PathBuf,
	},
};

use anyhow::{
	Context,
	Result,
};
use clap::Parser;
use serde::Deserialize;

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:5532";
const DEFAULT_ASSETS_DIR: &str = "assets";
const DEFAULT_AUTH_STORE: &str = "auth.enc";
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 30;
const DEFAULT_KEEP_ALIVE_SECS: u64 = 5;

#[derive(Debug, Parser)]
#[command(name = "hightale-server", about = "Hightale server")]
struct CliOptions {
	#[arg(long)]
	config: Option<PathBuf>,

	#[arg(long)]
	bind_addr: Option<SocketAddr>,

	#[arg(long)]
	assets_dir: Option<PathBuf>,

	#[arg(long)]
	quic_idle_timeout_secs: Option<u64>,

	#[arg(long)]
	quic_keep_alive_secs: Option<u64>,

	#[arg(long)]
	auth_session_token: Option<String>,

	#[arg(long)]
	auth_identity_token: Option<String>,

	#[arg(long)]
	auth_store_path: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
struct FileOptions {
	bind_addr: Option<SocketAddr>,
	assets_dir: Option<PathBuf>,
	quic_idle_timeout_secs: Option<u64>,
	quic_keep_alive_secs: Option<u64>,
	auth_session_token: Option<String>,
	auth_identity_token: Option<String>,
	auth_store_path: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
struct EnvOptions {
	#[serde(rename = "BIND_ADDR")]
	bind_addr: Option<SocketAddr>,
	#[serde(rename = "ASSETS_DIR")]
	assets_dir: Option<PathBuf>,
	#[serde(rename = "QUIC_IDLE_TIMEOUT_SECS")]
	quic_idle_timeout_secs: Option<u64>,
	#[serde(rename = "QUIC_KEEP_ALIVE_SECS")]
	quic_keep_alive_secs: Option<u64>,
	#[serde(rename = "AUTH_SESSION_TOKEN")]
	auth_session_token: Option<String>,
	#[serde(rename = "AUTH_IDENTITY_TOKEN")]
	auth_identity_token: Option<String>,
	#[serde(rename = "AUTH_STORE_PATH")]
	auth_store_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ServerOptions {
	pub bind_addr: SocketAddr,
	pub assets_dir: PathBuf,
	pub quic_idle_timeout_secs: u64,
	pub quic_keep_alive_secs: u64,
	pub auth_session_token: Option<String>,
	pub auth_identity_token: Option<String>,
	pub auth_store_path: PathBuf,
	pub config_path: Option<PathBuf>,
}

impl ServerOptions {
	pub fn load() -> Result<Self> {
		// Priority:
		// CLI Args -> Config File -> Env Variables
		let cli = CliOptions::parse();
		let config_path = cli.config.or_else(default_config_path);
		let file = match &config_path {
			Some(path) => read_config(path)?,
			None => FileOptions::default(),
		};
		let env = read_env().unwrap_or_default();

		let bind_addr = cli
			.bind_addr
			.or(file.bind_addr)
			.or(env.bind_addr)
			.unwrap_or_else(|| DEFAULT_BIND_ADDR.parse().expect("Default bind address is valid"));
		let assets_dir = cli.assets_dir.or(file.assets_dir).or(env.assets_dir).unwrap_or_else(|| PathBuf::from(DEFAULT_ASSETS_DIR));
		let quic_idle_timeout_secs = cli
			.quic_idle_timeout_secs
			.or(file.quic_idle_timeout_secs)
			.or(env.quic_idle_timeout_secs)
			.unwrap_or(DEFAULT_IDLE_TIMEOUT_SECS);
		let quic_keep_alive_secs = cli.quic_keep_alive_secs.or(file.quic_keep_alive_secs).or(env.quic_keep_alive_secs).unwrap_or(DEFAULT_KEEP_ALIVE_SECS);
		let auth_session_token = normalize_string_opt(cli.auth_session_token.or(file.auth_session_token).or(env.auth_session_token));
		let auth_identity_token = normalize_string_opt(cli.auth_identity_token.or(file.auth_identity_token).or(env.auth_identity_token));
		let auth_store_path = cli
			.auth_store_path
			.or(file.auth_store_path)
			.or(env.auth_store_path)
			.unwrap_or_else(|| PathBuf::from(DEFAULT_AUTH_STORE));

		Ok(Self {
			bind_addr,
			assets_dir,
			quic_idle_timeout_secs,
			quic_keep_alive_secs,
			auth_session_token,
			auth_identity_token,
			auth_store_path,
			config_path,
		})
	}
}

fn default_config_path() -> Option<PathBuf> {
	let path = Path::new("server.toml");
	if path.is_file() { Some(path.to_path_buf()) } else { None }
}

fn read_config(path: &Path) -> Result<FileOptions> {
	let raw = std::fs::read_to_string(path).with_context(|| format!("Reading config: {}", path.display()))?;
	let opts = toml::from_str::<FileOptions>(&raw).with_context(|| format!("Parsing config: {}", path.display()))?;
	Ok(opts)
}

fn read_env() -> Result<EnvOptions> {
	let env = envy::prefixed("HYTALE_").from_env::<EnvOptions>()?;
	Ok(env)
}

fn normalize_string_opt(value: Option<String>) -> Option<String> {
	value.and_then(|v| {
		let trimmed = v.trim();
		if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
	})
}
