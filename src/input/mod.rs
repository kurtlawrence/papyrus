use linefeed::terminal::DefaultTerminal;
use linefeed::{Interface, ReadResult};
use syn::{self, Expr, Item};

mod parse;
#[cfg(test)]
mod tests;

pub use self::parse::parse_program;

/// Reads input from `stdin`.
pub struct InputReader {
	buffer: String,
	interface: Interface<DefaultTerminal>,
}

/// Possible results from reading input from `InputReader`
#[derive(Debug, PartialEq)]
pub enum InputResult {
	/// Command argument as `(command_name, rest_of_line)`.
	Command(String, String),
	/// Code as input
	Program(Input),
	/// An empty line
	Empty,
	/// Needs more input; i.e. there is an unclosed delimiter.
	More,
	/// End of file reached.
	Eof,
	/// Error while parsing input.
	InputError(String),
	/// If part of the syntax tree has not yet been handled, an error will be shown.
	/// It is encouraged to submit an issue on [github](https://github.com/kurtlawrence/papyrus/issues).
	UnimplementedError(String),
}

/// Represents an input program.
#[derive(Debug, PartialEq)]
pub struct Input {
	/// Module-level items (`fn`, `enum`, `type`, `struct`, etc.)
	items: Vec<String>,
	/// Inner statements and declarations.
	stmts: Vec<Statement>,
}

/// Represents an inner statement.
#[derive(Debug, PartialEq)]
pub struct Statement {
	/// The code, not including the trailing semi if there is one.
	expr: String,
	/// Flags whether there is a trailing semi.
	semi: bool,
}

impl InputReader {
	/// Constructs a new `InputReader` reading from `stdin`.
	pub fn new(app_name: &'static str) -> Result<Self, String> {
		let r = match Interface::new(app_name) {
			Ok(r) => r,
			Err(e) => return Err(format!("failed to initialise interface: {}", e)),
		};
		Ok(InputReader {
			buffer: String::new(),
			interface: r,
		})
	}

	/// Reads a single command, item, or statement from `stdin`.
	/// Returns `More` if further input is required for a complete result.
	/// In this case, the input received so far is buffered internally.
	pub fn read_input(&mut self, prompt: &str) -> InputResult {
		// read the line
		let mut reader = self.interface.lock_reader();
		let line = {
			reader.set_prompt(prompt).unwrap();
			let r = match reader.read_line().ok().unwrap_or(ReadResult::Eof) {
				ReadResult::Eof => return InputResult::Eof,
				ReadResult::Input(s) => s,
				ReadResult::Signal(_) => {
					self.buffer.clear();
					return InputResult::Empty;
				}
			};
			r
		};

		self.buffer.push_str(&line);

		if self.buffer.is_empty() {
			return InputResult::Empty;
		}

		reader.add_history(line.to_owned());

		let res = if is_command(&self.buffer) {
			parse_command(&self.buffer)
		} else {
			parse_program(&self.buffer)
		};

		match res {
			InputResult::More => (),
			_ => self.buffer.clear(),
		};

		res
	}
}

pub fn is_command(line: &str) -> bool {
	line.starts_with(".") && !line.starts_with("..")
}

/// Parses a line of input as a command.
/// Returns either a `Command` value or an `InputError` value.
pub fn parse_command(line: &str) -> InputResult {
	if !is_command(line) {
		return InputResult::InputError("command must begin with `.` or `:`".to_string());
	}

	let line = &line[1..];
	let mut words = line.trim_right().splitn(2, ' ');

	match words.next() {
		Some(name) if !name.is_empty() => {
			InputResult::Command(name.to_string(), words.next().unwrap_or(&"").to_string())
		}
		_ => InputResult::InputError("expected command name".to_string()),
	}
}
