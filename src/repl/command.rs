use super::*;

type CommandAction<T, Arg, Data> = for<'data> fn(
	Repl<'data, ManualPrint, T, Arg, Data>,
	&str,
) -> Result<Repl<'data, Print, T, Arg, Data>, ()>;

/// A command definition.
pub struct Command<Term: Terminal, Arg, Data> {
	/// The command name.
	pub name: &'static str,
	/// Arguments expected type.
	pub arg_type: CmdArgs,
	/// Help string.
	pub help: &'static str,
	/// Action to take.
	pub action: CommandAction<Term, Arg, Data>,
}

/// Command arguments variants.
#[derive(Clone, PartialEq, Debug)]
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

impl<Term: Terminal, Arg, Data> Command<Term, Arg, Data> {
	/// Create a new `Command`.
	pub fn new(
		name: &'static str,
		arg_type: CmdArgs,
		help: &'static str,
		action: CommandAction<Term, Arg, Data>,
	) -> Self {
		Command {
			name: name,
			arg_type: arg_type,
			help: help,
			action: action,
		}
	}
}

impl<Term: Terminal, Arg, Data> Clone for Command<Term, Arg, Data> {
	fn clone(&self) -> Self {
		Command {
			name: self.name,
			arg_type: self.arg_type.clone(),
			help: self.help,
			action: self.action,
		}
	}
}

pub trait Commands<Term: Terminal, Arg, Data> {
	/// Builds the help string of the commands.
	fn build_help_response(&self, command: Option<&str>) -> String;
	/// Conveniance function to lookup the Commands and return if found.
	fn find_command(&self, command: &str) -> Result<Command<Term, Arg, Data>, String>;
}

impl<Term: Terminal, Arg, Data> Commands<Term, Arg, Data> for Vec<Command<Term, Arg, Data>> {
	fn build_help_response(&self, command: Option<&str>) -> String {
		let mut ret = String::new();

		let write_cmd_line = |cmd: &Command<Term, Arg, Data>, str_builder: &mut String| {
			str_builder.push_str(&cmd.name);

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
			match self.find_command(cmd) {
				Err(e) => ret.push_str(&e),
				Ok(cmd) => write_cmd_line(&cmd, &mut ret),
			}
		} else {
			ret.push_str("Available commands:\n");
			self.iter().for_each(|cmd| write_cmd_line(cmd, &mut ret));
		}
		ret
	}

	fn find_command(&self, command: &str) -> Result<Command<Term, Arg, Data>, String> {
		match self.iter().find(|c| c.name == command) {
			None => Err(format!("unrecognized command: {}", command)),
			Some(cmd) => Ok(cmd.clone()),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use linefeed::memory::MemoryTerminal;

	#[test]
	fn test_clone() {
		let c = Command::<MemoryTerminal>::new("test", CmdArgs::None, "Test help", |_, _| Err(()));
		let clone = c.clone();
		assert_eq!(c.name, clone.name);
		assert_eq!(c.arg_type, clone.arg_type);
		assert_eq!(c.help, clone.help);
	}

	#[test]
	fn help_response_types() {
		let cmds = vec![Command::<MemoryTerminal>::new(
			"test",
			CmdArgs::None,
			"Test help",
			|_, _| Err(()),
		)];
		assert_eq!(
			cmds.build_help_response(Some("test")),
			String::from("test Test help\n")
		);
		let cmds = vec![Command::<MemoryTerminal>::new(
			"test",
			CmdArgs::Text,
			"Test help",
			|_, _| Err(()),
		)];
		assert_eq!(
			cmds.build_help_response(Some("test")),
			String::from("test [text] Test help\n")
		);
		let cmds = vec![Command::<MemoryTerminal>::new(
			"test",
			CmdArgs::Filename,
			"Test help",
			|_, _| Err(()),
		)];
		assert_eq!(
			cmds.build_help_response(Some("test")),
			String::from("test <filename> Test help\n")
		);
		let cmds = vec![Command::<MemoryTerminal>::new(
			"test",
			CmdArgs::Expr,
			"Test help",
			|_, _| Err(()),
		)];
		assert_eq!(
			cmds.build_help_response(Some("test")),
			String::from("test <expr> Test help\n")
		);
		assert_eq!(
			cmds.build_help_response(None),
			String::from("Available commands:\ntest <expr> Test help\n")
		);
	}

	#[test]
	fn find_cmd() {
		let cmds = vec![Command::<MemoryTerminal>::new(
			"test",
			CmdArgs::None,
			"Test help",
			|_, _| Err(()),
		)];
		let cmd = cmds.find_command("test").unwrap();

		assert_eq!(cmd.name, "test");
		assert_eq!(cmd.arg_type, CmdArgs::None);
		assert_eq!(cmd.help, "Test help");

		match cmds.find_command("no-cmd") {
			Ok(_) => panic!("this should have failed"),
			Err(s) => assert_eq!(s, String::from("unrecognized command: no-cmd")),
		}
	}
}
