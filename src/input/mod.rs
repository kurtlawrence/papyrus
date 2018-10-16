use linefeed::terminal::DefaultTerminal;
use linefeed::{Interface, ReadResult};
use syn::{self, Expr, Item};

#[cfg(test)]
mod tests;

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
pub enum Input {
	/// Module-level items (`fn`, `enum`, `type`, `struct`, etc.)
	Item(String),
	/// Inner statements and declarations.
	/// The bool flags whether there was a trailing semicolon.
	Statements(Vec<String>, bool),
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

/// Parses a line of input as a program.
pub fn parse_program(code: &str) -> InputResult {
	debug!("parse program: {}", code);

	match syn::parse_str::<Item>(code) {
		Ok(item) => {
			return match item {
				Item::ExternCrate(_) => {
					error!("haven't handled item variant ExternCrate");
					InputResult::UnimplementedError("haven't handled item variant ExternCrate. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Use(_) => {
					error!("haven't handled item variant Use");
					InputResult::UnimplementedError("haven't handled item variant Use. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Static(_) => {
					error!("haven't handled item variant Static");
					InputResult::UnimplementedError("haven't handled item variant Static. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Const(_) => {
					error!("haven't handled item variant Const");
					InputResult::UnimplementedError("haven't handled item variant Const. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Fn(_) => InputResult::Program(Input::Item(code.to_string())),
				Item::Mod(_) => {
					error!("haven't handled item variant Mod");
					InputResult::UnimplementedError("haven't handled item variant Mod. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::ForeignMod(_) => {
					error!("haven't handled item variant ForeignMod");
					InputResult::UnimplementedError("haven't handled item variant ForeignMod. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Type(_) => {
					error!("haven't handled item variant Type");
					InputResult::UnimplementedError("haven't handled item variant Type. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Existential(_) => {
					error!("haven't handled item variant Existential");
					InputResult::UnimplementedError("haven't handled item variant Existential. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Struct(_) => {
					error!("haven't handled item variant Struct");
					InputResult::UnimplementedError("haven't handled item variant Struct. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Enum(_) => {
					error!("haven't handled item variant Enum");
					InputResult::UnimplementedError("haven't handled item variant Enum. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Union(_) => {
					error!("haven't handled item variant Union");
					InputResult::UnimplementedError("haven't handled item variant Union. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Trait(_) => {
					error!("haven't handled item variant Trait");
					InputResult::UnimplementedError("haven't handled item variant Trait. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::TraitAlias(_) => {
					error!("haven't handled item variant TraitAlias");
					InputResult::UnimplementedError("haven't handled item variant TraitAlias. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Impl(_) => {
					error!("haven't handled item variant Impl");
					InputResult::UnimplementedError("haven't handled item variant Impl. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Macro(_) => {
					error!("haven't handled item variant Macro");
					InputResult::UnimplementedError("haven't handled item variant Macro. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Macro2(_) => {
					error!("haven't handled item variant Macro2");
					InputResult::UnimplementedError("haven't handled item variant Macro2. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Verbatim(_) => {
					error!("haven't handled item variant Verbatim");
					InputResult::UnimplementedError("haven't handled item variant Verbatim. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
			};
		}
		Err(e) => if e.to_string().contains("LexError") {
			return InputResult::More;
		},
	}

	// not an item! loop through split statements (split on ;)
	let code = code.trim();
	let last = code.chars().last();
	if last.is_none() {
		return InputResult::Empty;
	}
	let last_is_semi = code.chars().last().unwrap() == ';';
	let mut stmts: Vec<String> = Vec::new();
	for stmt in code.split(';') {
		match syn::parse_str::<Expr>(stmt) {
			Ok(expr) => match expr {
				Expr::Box(_) => {
					error!("haven't handled expr variant Box");
					return InputResult::UnimplementedError("haven't handled expr variant Box. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::InPlace(_) => {
					error!("haven't handled expr variant InPlace");
					return	InputResult::UnimplementedError("haven't handled expr variant InPlace. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Array(_) => {
					error!("haven't handled expr variant Array");
					return		InputResult::UnimplementedError("haven't handled expr variant Array. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Call(_) => {
					error!("haven't handled expr variant Call");
					return		InputResult::UnimplementedError("haven't handled expr variant Call. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::MethodCall(_) => {
					error!("haven't handled expr variant MethodCall");
					return		InputResult::UnimplementedError("haven't handled expr variant MethodCall. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Tuple(_) => {
					error!("haven't handled expr variant Tuple");
					return			InputResult::UnimplementedError("haven't handled expr variant Tuple. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Binary(_) => stmts.push(stmt.to_string()),
				Expr::Unary(_) => {
					error!("haven't handled expr variant Unary");
					return			InputResult::UnimplementedError("haven't handled expr variant Unary. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Lit(_) => {
					error!("haven't handled expr variant Lit");
					return		InputResult::UnimplementedError("haven't handled expr variant Lit. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Cast(_) => {
					error!("haven't handled expr variant Cast");
					return			InputResult::UnimplementedError("haven't handled expr variant Cast. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Type(_) => {
					error!("haven't handled expr variant Type");
					return			InputResult::UnimplementedError("haven't handled expr variant Type. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Let(_) => {
					error!("haven't handled expr variant Let");
					return			InputResult::UnimplementedError("haven't handled expr variant Let. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::If(_) => {
					error!("haven't handled expr variant If");
					return			InputResult::UnimplementedError("haven't handled expr variant If. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::While(_) => {
					error!("haven't handled expr variant While");
					return			InputResult::UnimplementedError("haven't handled expr variant While. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::ForLoop(_) => {
					error!("haven't handled expr variant ForLoop");
					return		InputResult::UnimplementedError("haven't handled expr variant ForLoop. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Loop(_) => {
					error!("haven't handled expr variant For");
					return			InputResult::UnimplementedError("haven't handled expr variant For. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Match(_) => {
					error!("haven't handled expr variant Match");
					return			InputResult::UnimplementedError("haven't handled expr variant Match. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Closure(_) => {
					error!("haven't handled expr variant Closure");
					return		InputResult::UnimplementedError("haven't handled expr variant Closure. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Unsafe(_) => {
					error!("haven't handled expr variant Unsafe");
					return			InputResult::UnimplementedError("haven't handled expr variant Unsafe. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Block(_) => {
					error!("haven't handled expr variant Block");
					return			InputResult::UnimplementedError("haven't handled expr variant Block. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Assign(_) => {
					error!("haven't handled expr variant Assign");
					return		InputResult::UnimplementedError("haven't handled expr variant Assign. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::AssignOp(_) => {
					error!("haven't handled expr variant AssignOp");
					return			InputResult::UnimplementedError("haven't handled expr variant AssignOp. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Field(_) => {
					error!("haven't handled expr variant Field");
					return			InputResult::UnimplementedError("haven't handled expr variant Field. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Index(_) => {
					error!("haven't handled expr variant Index");
					return			InputResult::UnimplementedError("haven't handled expr variant Index. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Range(_) => {
					error!("haven't handled expr variant Range");
					return			InputResult::UnimplementedError("haven't handled expr variant Range. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Path(_) => {
					error!("haven't handled expr variant Path");
					return			InputResult::UnimplementedError("haven't handled expr variant Path. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Reference(_) => {
					error!("haven't handled expr variant Reference");
					return			InputResult::UnimplementedError("haven't handled expr variant Reference. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Break(_) => {
					error!("haven't handled expr variant Break");
					return		InputResult::UnimplementedError("haven't handled expr variant Break. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Continue(_) => {
					error!("haven't handled expr variant Continue");
					return			InputResult::UnimplementedError("haven't handled expr variant Continue. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Return(_) => {
					error!("haven't handled expr variant Return");
					return		InputResult::UnimplementedError("haven't handled expr variant Return. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Macro(_) => {
					error!("haven't handled expr variant Macro");
					return			InputResult::UnimplementedError("haven't handled expr variant Macro. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Struct(_) => {
					error!("haven't handled expr variant Struct");
					return		InputResult::UnimplementedError("haven't handled expr variant Struct. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Repeat(_) => {
					error!("haven't handled expr variant Repeat");
					return			InputResult::UnimplementedError("haven't handled expr variant Repeat. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Paren(_) => {
					error!("haven't handled expr variant Paren");
					return		InputResult::UnimplementedError("haven't handled expr variant Paren. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Group(_) => {
					error!("haven't handled expr variant Group");
					return		InputResult::UnimplementedError("haven't handled expr variant Group. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Try(_) => {
					error!("haven't handled expr variant Try");
					return		InputResult::UnimplementedError("haven't handled expr variant Try. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Async(_) => {
					error!("haven't handled expr variant Async");
					return			InputResult::UnimplementedError("haven't handled expr variant Async. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::TryBlock(_) => {
					error!("haven't handled expr variant TryBlock");
					return		InputResult::UnimplementedError("haven't handled expr variant TryBlock. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Yield(_) => {
					error!("haven't handled expr variant Yield");
					return		InputResult::UnimplementedError("haven't handled expr variant Yield. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
				Expr::Verbatim(_) => {
					error!("haven't handled expr variant Verbatim");
					return		InputResult::UnimplementedError("haven't handled expr variant Verbatim. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string());
				}
			},
			Err(e) => return InputResult::InputError(format!("{}", e)),
		};
	}

	InputResult::Program(Input::Statements(stmts, last_is_semi))
}
