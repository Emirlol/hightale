use std::any::Any;

use command::CommandSender;
use tracing::{
	error,
	info,
};

pub struct ConsoleSender;

impl CommandSender for ConsoleSender {
	fn send_message(&self, msg: &str) {
		info!("{}", msg);
	}

	fn send_error(&self, msg: &str) {
		error!("{}", msg);
	}

	fn has_permission(&self, _node: &str) -> bool {
		true // Console has all permissions
	}

	fn name(&self) -> &str {
		"Console"
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
}
