use super::*;
use std::io::{Result, Write};

impl<'a, Term: Terminal> Write for Writer<'a, Term> {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.0.write(&String::from_utf8_lossy(buf)).unwrap();
		Ok(buf.len())
	}

	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}

impl<'a, Term: Terminal> Writer<'a, Term> {
	pub fn overwrite_current_console_line(&mut self, line: &str) -> Result<()> {
		self.0.move_to_first_column()?;
		self.0.clear_to_screen_end()?;
		self.0.write(line)
	}
}
