use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use thiserror::Error;

use crate::{
	context::{
		CommandContext,
		CommandSender,
	},
	node::CommandNode,
};

pub struct CommandRegistry {
	root: CommandNode,
}

impl Default for CommandRegistry {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Error)]
pub enum CommandError {
	#[error("Unknown command '{name}'")]
	UnknownCommand { name: String },
	#[error("Invalid argument '{name}': {reason}")]
	InvalidArgument { name: String, reason: anyhow::Error },
	#[error("Command execution failed: {source}")]
	ExecutorError {
		#[from]
		source: anyhow::Error,
	},
	#[error("Incomplete command")]
	IncompleteCommand,
}

impl CommandRegistry {
	pub fn new() -> Self {
		Self { root: CommandNode::new() }
	}

	/// Entry point for the macro to register a root command
	pub fn register<F>(&mut self, name: &str, builder: F)
	where
		F: FnOnce(&mut CommandNode),
	{
		let root_cmd = self.root.children.entry(name.to_string()).or_default();
		builder(root_cmd);
	}

	pub fn execute(&self, sender: Arc<dyn CommandSender>, input: &str) -> Result<(), CommandError> {
		let parts: Vec<&str> = input.split_whitespace().collect();
		if parts.is_empty() {
			return Ok(());
		}

		let mut current_node = &self.root;
		let mut parsed_args = HashMap::new();
		let mut cursor = 0;

		while cursor < parts.len() {
			let part = parts[cursor];

			if let Some(child) = current_node.children.get(part) {
				current_node = child;
				cursor += 1;
				continue;
			}

			let mut matched = false;
			let mut last_err: Option<anyhow::Error> = None;
			for arg in &current_node.arguments {
				match arg.parser.parse(part) {
					Ok(parsed_value) => {
						parsed_args.insert(arg.name.clone(), parsed_value);

						current_node = &arg.node;
						matched = true;
						cursor += 1;
						break;
					}
					Err(e) => last_err = Some(CommandError::InvalidArgument { name: arg.name.clone(), reason: e }.into()),
				}
			}

			if !matched {
				return Err(if let Some(err) = last_err {
					CommandError::InvalidArgument { name: part.to_string(), reason: err }
				} else {
					CommandError::UnknownCommand { name: part.to_string() }
				});
			}
		}

		if let Some(executor) = &current_node.executor {
			let ctx = CommandContext { sender, args: &parsed_args };
			executor(&ctx).map_err(|e| CommandError::ExecutorError { source: e })?;
			Ok(())
		} else {
			Err(CommandError::IncompleteCommand)
		}
	}

	pub fn get_suggestions(&self, input: &str) -> Vec<String> {
		let parts: Vec<&str> = input.split_whitespace().collect();

		let mut current_node = &self.root;
		let mut cursor = 0;

		while cursor < parts.len() {
			let part = parts[cursor];

			if cursor == parts.len() - 1 && !input.ends_with(' ') {
				break;
			}

			let mut found = false;
			if let Some(child) = current_node.children.get(part) {
				current_node = child;
				found = true;
			} else {
				for arg in &current_node.arguments {
					if arg.parser.parse(part).is_ok() {
						current_node = &arg.node;
						found = true;
						break;
					}
				}
			}

			if !found {
				return vec![];
			}
			cursor += 1;
		}

		let mut suggestions = Vec::new();
		let last_word = if input.ends_with(' ') { "" } else { parts.last().copied().unwrap_or("") };

		for literal in current_node.children.keys() {
			if literal.starts_with(last_word) {
				suggestions.push(literal.clone());
			}
		}

		for arg in &current_node.arguments {
			let arg_suggestions = arg.parser.suggestions();
			if !arg_suggestions.is_empty() {
				for s in arg_suggestions {
					if s.starts_with(last_word) {
						suggestions.push(s);
					}
				}
			} else {
				suggestions.push(format!("<{}>", arg.name));
			}
		}

		suggestions
	}
}
