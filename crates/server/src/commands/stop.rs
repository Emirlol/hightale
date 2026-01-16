use anyhow::anyhow;
use command::{command, CommandRegistry};

pub fn register(registry: &mut CommandRegistry, shutdown_tx: tokio::sync::mpsc::UnboundedSender<()>) {
	command!(registry, "stop", {
		executes move |_| {
			shutdown_tx.send(()).map_err(|e| anyhow!("Failed to send shutdown signal: {}", e))
		}
	});
}