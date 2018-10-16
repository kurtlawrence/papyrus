use super::*;

/// A command definition.
pub struct Command<'r> {
	/// The command name.
	pub name: &'static str,
	/// Arguments expected type.
	pub arg_type: CmdArgs,
	/// Help string.
	pub help: &'static str,
	/// Action to take.
	pub action: Box<Fn(&Repl, &str) + 'r>,
}

/// Command arguments variants.
pub enum CmdArgs {
	/// No arguments.
	None,
	/// Command accepts a local filename.
	Filename,
	/// Optional unprocessed text may be accepted.
	Text,
	/// A Rust expression.
	Expr,
}

impl<'r> Command<'r> {
	pub fn new<A: 'r + Fn(&Repl, &str)>(
		name: &'static str,
		arg_type: CmdArgs,
		help: &'static str,
		action: A,
	) -> Self {
		Command {
			name: name,
			arg_type: arg_type,
			help: help,
			action: Box::new(action) as Box<Fn(&Repl, &str) + 'r>,
		}
	}
}

pub trait Commands {
	/// Builds the help string of the commands.
	fn build_help_response(&self, command: Option<&str>) -> String;
}

impl<'r> Commands for Vec<Command<'r>> {
	fn build_help_response(&self, command: Option<&str>) -> String {
		let mut ret = String::new();

		let write_cmd_line = |cmd: &Command, str_builder: &mut String| {
			str_builder.push_str(cmd.name);

			match cmd.arg_type {
				CmdArgs::None => (),
				CmdArgs::Filename => str_builder.push_str(" <filename>"),
				CmdArgs::Text => str_builder.push_str(" [text]"),
				CmdArgs::Expr => str_builder.push_str(" <expr>"),
			}
			str_builder.push(' ');
			str_builder.push_str(cmd.help);
			str_builder.push('\n');
		};

		if let Some(cmd) = command {
			match self.iter().find(|c| c.name == cmd) {
				None => ret.push_str(&format!("unrecognized command: {}", cmd)),
				Some(cmd) => write_cmd_line(cmd, &mut ret),
			}
		} else {
			ret.push_str("Available commands:\n");
			self.iter().for_each(|cmd| write_cmd_line(cmd, &mut ret));
		}
		ret
	}
}
