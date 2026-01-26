pub mod sender;
mod writer;

use std::sync::{
	Arc,
	RwLock,
};

use command::CommandRegistry;
use rustyline::{
	completion::{
		Completer,
		Pair,
	},
	highlight::Highlighter,
	hint::Hinter,
	validate::Validator,
	Config,
	Context,
	Editor,
	Helper,
	Result as RlResult,
};
use tokio::sync::mpsc;
use tracing::{
	error,
	info,
};

use crate::console::{
	sender::ConsoleSender,
	writer::RustylineLogWriter,
};

#[derive(Clone)]
pub struct ServerHelper {
	// We use std::sync::RwLock because Rustyline is synchronous/blocking
	pub registry: Arc<RwLock<CommandRegistry>>,
}
impl Completer for ServerHelper {
	type Candidate = Pair;

	fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> RlResult<(usize, Vec<Pair>)> {
		let registry = self.registry.read().unwrap();
		// TODO: Improve parsing to respect cursor position (quotes/escapes) for better tab completion.
		let prefix = &line[..pos];
		let suggestions = registry.get_suggestions(prefix);

		let pairs = suggestions.into_iter().map(|s| Pair { display: s.clone(), replacement: s }).collect();

		let start_pos = prefix.rfind(' ').map(|i| i + 1).unwrap_or(0);

		Ok((start_pos, pairs))
	}
}
impl Hinter for ServerHelper {
	type Hint = String;

	fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
		None // Maybe later.
	}
}
impl Highlighter for ServerHelper {}
impl Validator for ServerHelper {}
impl Helper for ServerHelper {}

pub fn setup_interactive(registry: Arc<RwLock<CommandRegistry>>, shutdown_tx: mpsc::UnboundedSender<()>) -> anyhow::Result<(RustylineLogWriter, impl FnOnce() + Send + 'static)> {
	let config = Config::builder().auto_add_history(true).build();

	let mut editor = Editor::<ServerHelper, _>::with_history(config, rustyline::history::FileHistory::new())?;
	editor.set_helper(Some(ServerHelper { registry: registry.clone() }));

	let printer = editor.create_external_printer()?;
	let log_writer = RustylineLogWriter::new(Box::new(printer));

	let loop_task = move || {
		let sender = Arc::new(ConsoleSender);
		info!("Console Ready. Type 'help' for commands.");

		loop {
			let readline = editor.readline("> ");

			match readline {
				Ok(line) => {
					let line = line.trim();
					if line.is_empty() {
						continue;
					}

					// Execute Command
					let reg_lock = registry.read().unwrap();
					if let Err(e) = reg_lock.execute(sender.clone(), line) {
						error!("Error: {}", e);
					}
				}
				Err(rustyline::error::ReadlineError::Interrupted) => {
					let _ = shutdown_tx.send(());
					break;
				}
				Err(rustyline::error::ReadlineError::Eof) => {
					let _ = shutdown_tx.send(());
					break;
				}
				Err(err) => {
					error!("Console Error: {:?}", err);
					break;
				}
			}
		}
	};

	Ok((log_writer, loop_task))
}
