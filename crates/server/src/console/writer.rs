use std::{
	io,
	sync::Arc,
};

use parking_lot::Mutex;
use rustyline::ExternalPrinter;

#[derive(Clone)]
pub struct RustylineLogWriter {
	printer: Arc<Mutex<Box<dyn ExternalPrinter + Send>>>,
}

impl RustylineLogWriter {
	pub fn new(printer: Box<dyn ExternalPrinter + Send>) -> Self {
		Self {
			printer: Arc::new(Mutex::new(printer)),
		}
	}
}

impl io::Write for RustylineLogWriter {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		// Lossy to prevent crashing on weird log bytes
		let msg = String::from_utf8_lossy(buf);

		self.printer.lock().print(msg.to_string()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(()) // ExternalPrinter handles flushing
	}
}
