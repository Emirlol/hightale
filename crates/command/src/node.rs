use std::collections::HashMap;

use anyhow::Result;

use crate::{
	args::ArgParser,
	context::CommandContext,
};

// An Executor is a closure that takes Context and returns a Result
pub type CommandExecutor = Box<dyn Fn(&CommandContext) -> Result<()> + Send + Sync>;

pub struct CommandNode {
	pub executor: Option<CommandExecutor>,
	pub children: HashMap<String, CommandNode>,
	pub arguments: Vec<ArgumentNode>,
	pub help: String,
}

pub struct ArgumentNode {
	pub name: String,
	pub parser: Box<dyn ArgParser>,
	pub node: CommandNode,
}

impl Default for CommandNode {
	fn default() -> Self {
		Self::new()
	}
}

impl CommandNode {
	pub fn new() -> Self {
		Self {
			executor: None,
			children: HashMap::new(),
			arguments: Vec::new(),
			help: String::new(),
		}
	}

	/// DSL Helper: Create/Get a literal child and configure it via closure
	pub fn literal<F>(&mut self, name: &str, builder: F) -> &mut Self
	where
		F: FnOnce(&mut CommandNode),
	{
		let child = self.children.entry(name.to_string()).or_default();
		builder(child);
		self
	}

	/// DSL Helper: Create an argument child and configure it via closure
	pub fn argument<F>(&mut self, name: &str, parser: Box<dyn ArgParser>, builder: F) -> &mut Self
	where
		F: FnOnce(&mut CommandNode),
	{
		let mut arg_node = ArgumentNode {
			name: name.to_string(),
			parser,
			node: CommandNode::new(),
		};
		builder(&mut arg_node.node);
		self.arguments.push(arg_node);
		self
	}

	/// Sets the logic to run when this node is executed
	pub fn executes<F>(&mut self, f: F)
	where
		F: Fn(&CommandContext) -> Result<()> + Send + Sync + 'static,
	{
		self.executor = Some(Box::new(f));
	}
}
