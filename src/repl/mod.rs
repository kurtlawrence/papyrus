use super::input::{Input, InputReader, InputResult};
use super::*;
use super::file::Source;

mod command;

pub use self::command::{CmdArgs, Command, Commands};

/// A REPL instance.
pub struct Repl<'r> {
	/// The REPL handled commands.
	/// Can be extended.
	/// ```ignore
	/// let repl = Repl::new();
	/// repl.commands.push(Command::new("load", CmdArgs::Filename, "load and evaluate file contents as inputs", |r, arg_text| {
	/// 	r.run_file(arg_text);
	/// }));
	pub commands: Vec<Command<'r>>,
	/// Items compiled into every program. These are functions, types, etc.
	pub items: Vec<String>,
	/// Statements applied in order.
	pub statements: Vec<String>,
}

impl<'r> Repl<'r> {
	/// A new REPL instance.
	pub fn new() -> Self {
		let mut r = Repl {
			commands: Vec::new(),
			items: Vec::new(),
			statements: Vec::new(),
		};
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
		r.commands.push(Command::new(
			"load",
			CmdArgs::Filename,
			"load *.rs or *.rscript as inputs",
			|repl, arg| {},
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
		loop {
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
					// self.handle_command(name, args);
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
	fn handle_input(&mut self, mut input: Input, display: bool) {
		let mut items = self.items.join("\n");
		let mut statements = self.statements.join("\n");
		match input {
			Input::Item(ref code) => {
				items.push_str("\n");
				items.push_str(code);
			}
			Input::Statement(ref code, expr) => {
				statements.push_str("\n");
				let code = if expr {
					if display {
						format!(r#"println!("{{}}", {{ {} }});"#, code)
					} else {
						format!(r#"println!("{{:?}}", {{ {} }});"#, code)
					}
				} else {
					code.to_string()
				};
				statements.push_str(&code);
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

		let s = Script::build_compile_dir(&src, &"test",)
			.unwrap();
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

	// fn load_file<P: AsRef<path::Path>>(filename: &P) -> Input {

	// }
}
