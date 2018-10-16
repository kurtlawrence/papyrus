use super::file::Source;
use super::input::{self, Input, InputReader, InputResult};
use super::*;

mod command;

pub use self::command::{CmdArgs, Command, Commands};

/// A REPL instance.
pub struct Repl {
	/// The REPL handled commands.
	/// Can be extended.
	/// ```ignore
	/// let repl = Repl::new();
	/// repl.commands.push(Command::new("load", CmdArgs::Filename, "load and evaluate file contents as inputs", |r, arg_text| {
	/// 	r.run_file(arg_text);
	/// }));
	pub commands: Vec<Command>,
	/// Items compiled into every program. These are functions, types, etc.
	pub items: Vec<String>,
	/// Blocks of statements applied in order.
	pub statements: Vec<Vec<String>>,
	/// Flag whether to keep looping,
	exit_loop: bool,
}

impl Repl {
	/// A new REPL instance.
	pub fn new() -> Self {
		let mut r = Repl {
			commands: Vec::new(),
			items: Vec::new(),
			statements: Vec::new(),
			exit_loop: false,
		};
		// help
		r.commands.push(Command::new(
			"help",
			CmdArgs::Text,
			"Show help for commands",
			|repl, arg| {
				println!(
					"{}",
					repl.commands.build_help_response(if arg.is_empty() {
						None
					} else {
						Some(arg)
					})
				)
			},
		));
		// exit
		r.commands.push(Command::new(
			"exit",
			CmdArgs::Text,
			"Exit repl",
			|repl, _| repl.exit_loop = true,
		));
		// load
		r.commands.push(Command::new(
			"load",
			CmdArgs::Filename,
			"load *.rs or *.rscript as inputs",
			|repl, arg| {
				let res = match Source::load(&arg) {
					Ok(src) => input::parse_program(&src.src),
					Err(e) => InputResult::InputError(e),
				};
				match res {
					InputResult::Program(input) => {
						debug!("read program: {:?}", input);
						repl.handle_input(input, false);
					}
					InputResult::InputError(e) => println!("{}", e),
					_ => println!("haven't handled file input"),
				}
			},
		));
		r
	}

	/// Run the REPL interactively.
	///
	/// # Panics
	/// - Failure to initialise `InputReader`.
	/// - `app_name` or `prompt` is empty.
	pub fn run(mut self, app_name: &'static str, prompt: &str) {
		assert!(!prompt.is_empty());
		assert!(!app_name.is_empty());
		let mut input = InputReader::new(app_name).expect("failed to start input reader");
		let mut more = false;
		self.exit_loop = false;
		while !self.exit_loop {
			let prompt = if more {
				format!("{}.>", prompt)
			} else {
				format!("{}=>", prompt)
			};
			let res = input.read_input(&prompt);

			match res {
				InputResult::Command(name, args) => {
					debug!("read command: {} {:?}", name, args);
					more = false;
					match self.commands.find_command(&name) {
						Err(e) => println!("{}", e),
						Ok(cmd) => (cmd.action)(&mut self, &args),
					};
				}
				InputResult::Program(input) => {
					debug!("read program: {:?}", input);
					more = false;
					self.handle_input(input, false);
				}
				InputResult::Empty => (),
				InputResult::More => {
					more = true;
				}
				InputResult::Eof => break,
				InputResult::InputError(err) | InputResult::UnimplementedError(err) => {
					println!("{}", err);
					more = false;
				}
			};
		}
	}

	/// Runs a single program input.
	/// If `display` is `true`, an expression will be printed using the
	/// `Display` trait; otherwise, it is printed as `Debug`.
	fn handle_input(&mut self, input: Input, display: bool) {
		let mut items = self.items.join("\n");
		let mut statements = self
			.statements
			.iter()
			.map(|s| s.join("\n"))
			.collect::<Vec<_>>()
			.join("\n");
		let data_num = self.statements.len();
		match input {
			Input::Item(ref code) => {
				items.push_str("\n");
				items.push_str(code);
			}
			Input::Statements(ref code, trailing_semi) => {
				statements.push_str("\n");
				for stmt in code[0..code.len() - 1].iter() {
					statements.push_str(stmt);
					statements.push('\n');
				}
				let last_stmt = if code.len() == 0 {
					""
				} else {
					&code[code.len() - 1]
				};

				let last_stmt = if !trailing_semi {
					if display {
						format!(
							r#"let out{out_num} = {stmt};\nprintln!("{{}}", out{out_num});"#,
							out_num = data_num,
							stmt = last_stmt
						)
					} else {
						format!(
							r#"let out{out_num} = {stmt};\nprintln!("{{:?}}", out{out_num});"#,
							out_num = data_num,
							stmt = last_stmt
						)
					}
				} else {
					last_stmt.to_string()
				};
				statements.push_str(&last_stmt);
			}
		}

		let code = format!(
			r#"
pub fn main() {{
    let _ = std::panic::catch_unwind(_papyrus_inner);
}}

fn _papyrus_inner() {{
{stmts}
}}

{items}
"#,
			stmts = statements,
			items = items
		);

		let src = Source {
			src: code,
			file_name: String::from("mem-code"),
			file_type: SourceFileType::Rs,
			crates: Vec::new(),
		};

		let s = Script::build_compile_dir(&src, &"test").unwrap();
		match s.run(&::std::env::current_dir().unwrap()) {
			Ok(output) => {
				if output.status.success() {
					println!("{}", String::from_utf8_lossy(&output.stdout));
					if let Input::Item(c) = input {
						//Successful compile means we can add the new items to every program
						self.items.push(c);
					}
				} else {
					println!("{}", String::from_utf8_lossy(&output.stderr));
				}
			}
			Err(e) => println!("{}", e),
		}
	}
}
