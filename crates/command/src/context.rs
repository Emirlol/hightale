use std::{
	any::{
		Any,
		TypeId,
	},
	collections::HashMap,
};
use std::sync::Arc;
use anyhow::Result;
use thiserror::Error;

use crate::args::BoxedArg;

pub trait CommandSender: Send + Sync {
	fn send_message(&self, msg: &str);
	fn send_error(&self, msg: &str);
	fn has_permission(&self, permission_node: &str) -> bool;
	fn name(&self) -> &str;
	/// Allows downcasting to specific sender types (e.g., PlayerConnection)
	fn as_any(&self) -> &dyn Any;
}

pub struct CommandContext<'a> {
	pub sender: Arc<dyn CommandSender>, // The sender may outlive the context
	pub args: &'a HashMap<String, BoxedArg>,
}

#[derive(Debug, Error)]
pub enum ArgError {
	#[error("Argument '{name}' not found")]
	NotFound { name: String },
	#[error("Argument '{name}' is not of type {expected:?}")]
	WrongType { name: String, expected: TypeId },
}

impl ArgError {
	fn not_found(name: &str) -> Self {
		Self::NotFound { name: name.to_owned() }
	}

	fn wrong_type<T: 'static>(name: &str) -> Self {
		Self::WrongType {
			name: name.to_owned(),
			expected: TypeId::of::<T>(),
		}
	}
}

impl<'a> CommandContext<'a> {
	/// Retrieve a parsed argument.
	pub fn arg<T: 'static>(&self, name: &str) -> Result<&T, ArgError> {
		let val = self.args.get(name).ok_or_else(|| ArgError::not_found(name))?;
		val.downcast_ref::<T>().ok_or_else(|| ArgError::wrong_type::<T>(name))
	}
}
