use linefeed::terminal::DefaultTerminal;
use linefeed::{Interface, ReadResult, Reader};
use std::io;
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
	Command(String, Option<String>),
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
	Statement(String, bool),
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
	(line.starts_with(".") && !line.starts_with(".."))
		|| (line.starts_with(":") && !line.starts_with("::"))
}

/// Parses a line of input as a command.
/// Returns either a `Command` value or an `InputError` value.
pub fn parse_command(line: &str) -> InputResult {
	if !is_command(line) {
		return InputResult::InputError("command must begin with `.` or `:`".to_string());
	}

	let line = &line[1..];
	let mut words = line.trim_right().splitn(2, ' ');

	let name = match words.next() {
		Some(name) if !name.is_empty() => name,
		_ => return InputResult::InputError("expected command name".to_string()),
	};

	return InputResult::InputError("some error here".to_string());

	// let cmd = match lookup_command(name) {
	// 	Some(cmd) => cmd,
	// 	None => return InputResult::InputError(format!("unrecognized command: {}", name)),
	// };

	// let args = words.next();

	// match cmd.accepts {
	// 	CmdArgs::Nothing if args.is_some() => {
	// 		InputResult::InputError(format!("command `{}` takes no arguments", cmd.name))
	// 	}
	// 	CmdArgs::Expr if args.is_none() => {
	// 		InputResult::InputError(format!("command `{}` expects an expression", cmd.name))
	// 	}
	// 	CmdArgs::Expr => {
	// 		let args = args.unwrap();
	// 		match parse_program(args, filter, None) {
	// 			InputResult::Program(_) => {
	// 				InputResult::Command(name.to_owned(), Some(args.to_owned()))
	// 			}
	// 			i => i,
	// 		}
	// 	}
	// 	_ => InputResult::Command(name.to_owned(), args.map(|s| s.to_owned())),
	// }
}

/// Parses a line of input as a program.
pub fn parse_program(code: &str) -> InputResult {
	debug!("parse program: {}", code);

	match syn::parse_str::<Item>(code) {
		Ok(item) => {
			return match item {
				Item::ExternCrate(_) => {
					error!("haven't handled expr variant ExternCrate");
					InputResult::UnimplementedError("haven't handled expr variant ExternCrate. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Use(_) => {
					error!("haven't handled expr variant Use");
					InputResult::UnimplementedError("haven't handled expr variant Use. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Static(_) => {
					error!("haven't handled expr variant Static");
					InputResult::UnimplementedError("haven't handled expr variant Static. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Const(_) => {
					error!("haven't handled expr variant Const");
					InputResult::UnimplementedError("haven't handled expr variant Const. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Fn(_) => {
					error!("haven't handled expr variant Fn");
					InputResult::UnimplementedError("haven't handled expr variant Fn. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Mod(_) => {
					error!("haven't handled expr variant Mod");
					InputResult::UnimplementedError("haven't handled expr variant Mod. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::ForeignMod(_) => {
					error!("haven't handled expr variant ForeignMod");
					InputResult::UnimplementedError("haven't handled expr variant ForeignMod. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Type(_) => {
					error!("haven't handled expr variant Type");
					InputResult::UnimplementedError("haven't handled expr variant Type. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Existential(_) => {
					error!("haven't handled expr variant Existential");
					InputResult::UnimplementedError("haven't handled expr variant Existential. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Struct(_) => {
					error!("haven't handled expr variant Struct");
					InputResult::UnimplementedError("haven't handled expr variant Struct. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Enum(_) => {
					error!("haven't handled expr variant Enum");
					InputResult::UnimplementedError("haven't handled expr variant Enum. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Union(_) => {
					error!("haven't handled expr variant Union");
					InputResult::UnimplementedError("haven't handled expr variant Union. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Trait(_) => {
					error!("haven't handled expr variant Trait");
					InputResult::UnimplementedError("haven't handled expr variant Trait. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::TraitAlias(_) => {
					error!("haven't handled expr variant TraitAlias");
					InputResult::UnimplementedError("haven't handled expr variant TraitAlias. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Impl(_) => {
					error!("haven't handled expr variant Impl");
					InputResult::UnimplementedError("haven't handled expr variant Impl. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Macro(_) => {
					error!("haven't handled expr variant Macro");
					InputResult::UnimplementedError("haven't handled expr variant Macro. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Macro2(_) => {
					error!("haven't handled expr variant Macro2");
					InputResult::UnimplementedError("haven't handled expr variant Macro2. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
				Item::Verbatim(_) => {
					error!("haven't handled expr variant Verbatim");
					InputResult::UnimplementedError("haven't handled expr variant Verbatim. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
				}
			};
		}
		Err(e) => if e.to_string().contains("LexError") {
			return InputResult::More;
		},
	}

	match syn::parse_str::<Expr>(code) {
		Ok(expr) => match expr {
			Expr::Box(_) => {
				error!("haven't handled expr variant Box");
				InputResult::UnimplementedError("haven't handled expr variant Box. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::InPlace(_) => {
				error!("haven't handled expr variant InPlace");
				InputResult::UnimplementedError("haven't handled expr variant InPlace. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Array(_) => {
				error!("haven't handled expr variant Array");
				InputResult::UnimplementedError("haven't handled expr variant Array. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Call(_) => {
				error!("haven't handled expr variant Call");
				InputResult::UnimplementedError("haven't handled expr variant Call. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::MethodCall(_) => {
				error!("haven't handled expr variant MethodCall");
				InputResult::UnimplementedError("haven't handled expr variant MethodCall. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Tuple(_) => {
				error!("haven't handled expr variant Tuple");
				InputResult::UnimplementedError("haven't handled expr variant Tuple. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Binary(_) => InputResult::Program(Input::Statement(code.to_string(), true)),
			Expr::Unary(_) => {
				error!("haven't handled expr variant Unary");
				InputResult::UnimplementedError("haven't handled expr variant Unary. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Lit(_) => {
				error!("haven't handled expr variant Lit");
				InputResult::UnimplementedError("haven't handled expr variant Lit. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Cast(_) => {
				error!("haven't handled expr variant Cast");
				InputResult::UnimplementedError("haven't handled expr variant Cast. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Type(_) => {
				error!("haven't handled expr variant Type");
				InputResult::UnimplementedError("haven't handled expr variant Type. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Let(_) => {
				error!("haven't handled expr variant Let");
				InputResult::UnimplementedError("haven't handled expr variant Let. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::If(_) => {
				error!("haven't handled expr variant If");
				InputResult::UnimplementedError("haven't handled expr variant If. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::While(_) => {
				error!("haven't handled expr variant While");
				InputResult::UnimplementedError("haven't handled expr variant While. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::ForLoop(_) => {
				error!("haven't handled expr variant ForLoop");
				InputResult::UnimplementedError("haven't handled expr variant ForLoop. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Loop(_) => {
				error!("haven't handled expr variant For");
				InputResult::UnimplementedError("haven't handled expr variant For. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Match(_) => {
				error!("haven't handled expr variant Match");
				InputResult::UnimplementedError("haven't handled expr variant Match. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Closure(_) => {
				error!("haven't handled expr variant Closure");
				InputResult::UnimplementedError("haven't handled expr variant Closure. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Unsafe(_) => {
				error!("haven't handled expr variant Unsafe");
				InputResult::UnimplementedError("haven't handled expr variant Unsafe. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Block(_) => {
				error!("haven't handled expr variant Block");
				InputResult::UnimplementedError("haven't handled expr variant Block. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Assign(_) => {
				error!("haven't handled expr variant Assign");
				InputResult::UnimplementedError("haven't handled expr variant Assign. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::AssignOp(_) => {
				error!("haven't handled expr variant AssignOp");
				InputResult::UnimplementedError("haven't handled expr variant AssignOp. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Field(_) => {
				error!("haven't handled expr variant Field");
				InputResult::UnimplementedError("haven't handled expr variant Field. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Index(_) => {
				error!("haven't handled expr variant Index");
				InputResult::UnimplementedError("haven't handled expr variant Index. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Range(_) => {
				error!("haven't handled expr variant Range");
				InputResult::UnimplementedError("haven't handled expr variant Range. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Path(_) => {
				error!("haven't handled expr variant Path");
				InputResult::UnimplementedError("haven't handled expr variant Path. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Reference(_) => {
				error!("haven't handled expr variant Reference");
				InputResult::UnimplementedError("haven't handled expr variant Reference. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Break(_) => {
				error!("haven't handled expr variant Break");
				InputResult::UnimplementedError("haven't handled expr variant Break. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Continue(_) => {
				error!("haven't handled expr variant Continue");
				InputResult::UnimplementedError("haven't handled expr variant Continue. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Return(_) => {
				error!("haven't handled expr variant Return");
				InputResult::UnimplementedError("haven't handled expr variant Return. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Macro(_) => {
				error!("haven't handled expr variant Macro");
				InputResult::UnimplementedError("haven't handled expr variant Macro. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Struct(_) => {
				error!("haven't handled expr variant Struct");
				InputResult::UnimplementedError("haven't handled expr variant Struct. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Repeat(_) => {
				error!("haven't handled expr variant Repeat");
				InputResult::UnimplementedError("haven't handled expr variant Repeat. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Paren(_) => {
				error!("haven't handled expr variant Paren");
				InputResult::UnimplementedError("haven't handled expr variant Paren. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Group(_) => {
				error!("haven't handled expr variant Group");
				InputResult::UnimplementedError("haven't handled expr variant Group. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Try(_) => {
				error!("haven't handled expr variant Try");
				InputResult::UnimplementedError("haven't handled expr variant Try. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Async(_) => {
				error!("haven't handled expr variant Async");
				InputResult::UnimplementedError("haven't handled expr variant Async. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::TryBlock(_) => {
				error!("haven't handled expr variant TryBlock");
				InputResult::UnimplementedError("haven't handled expr variant TryBlock. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Yield(_) => {
				error!("haven't handled expr variant Yield");
				InputResult::UnimplementedError("haven't handled expr variant Yield. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
			Expr::Verbatim(_) => {
				error!("haven't handled expr variant Verbatim");
				InputResult::UnimplementedError("haven't handled expr variant Verbatim. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
			}
		},
		Err(e) => {
			if &code[code.len() - 1..code.len()] == ";" {
				InputResult::Program(Input::Statement(code.to_string(), false))
			} else {
				InputResult::InputError(format!("{:?}", e))
			}
		}
	}
}
