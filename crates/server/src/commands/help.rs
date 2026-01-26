use command::{
	command,
	CommandRegistry,
};

pub fn register(registry: &mut CommandRegistry) {
	command!(registry, "help", {
		executes move |ctx| {
			let sender = ctx.sender.clone();
			let commands = ctx.registry.root_commands();
			if commands.is_empty() {
				sender.send_message("No commands registered.");
			} else {
				sender.send_message("Available commands:");
				for cmd in commands {
					sender.send_message(&format!("- {}", cmd));
				}
			}
			Ok(())
		}
	});
}
