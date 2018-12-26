use super::compile::*;
use super::file::SourceFile;
use super::input::{self, Input, InputReader, InputResult};
use super::*;
use colored::*;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use term_cursor;

mod command;
#[cfg(test)]
mod tests;

use self::command::Commands;
pub use self::command::{CmdArgs, Command, CommandActionArgs};

/// A REPL instance.
pub struct Repl {
	/// Flag whether to keep looping.
	exit_loop: bool,
	/// The REPL handled commands.
	/// Can be extended.
	/// ```ignore
	/// let mut repl = Repl::new();
	/// repl.commands.push(Command::new("load", CmdArgs::Filename, "load and evaluate file contents as inputs", |args| {
	/// 	args.repl.run_file(args.arg);
	/// }));
	pub commands: Vec<Command>,
	/// Items compiled into every program. These are functions, types, etc.
	pub items: Vec<Vec<String>>,
	/// Blocks of statements applied in order.
	pub statements: Vec<Vec<String>>,
	/// Crates to referenced.
	pub crates: Vec<CrateType>,
	/// Flag whether to print to stdout.
	pub print: bool,
	/// App and prompt text.
	pub name: &'static str,
	/// The colour of the prompt region. ie `papyrus`.
	pub prompt_colour: Color,
	/// The colour of the out component. ie `[out0]`.
	pub out_colour: Color,
}

#[derive(Clone)]
struct Additional {
	items: Option<Vec<String>>,
	stmts: Option<AdditionalStatements>,
	crates: Vec<CrateType>,
}

#[derive(Clone)]
struct AdditionalStatements {
	stmts: Vec<String>,
	print_stmt: String,
}

impl Repl {
	/// A new REPL instance.
	pub fn new() -> Self {
		let mut r = Repl {
			commands: Vec::new(),
			items: Vec::new(),
			statements: Vec::new(),
			crates: Vec::new(),
			exit_loop: false,
			name: "papyrus",
			prompt_colour: Color::Cyan,
			out_colour: Color::BrightGreen,
			print: true,
		};
		// help
		r.commands.push(Command::new(
			"help",
			CmdArgs::Text,
			"Show help for commands",
			|args| {
				let (repl, arg) = { (args.repl, args.arg) };
				// colour output
				let output = repl.commands.build_help_response(if arg.is_empty() {
					None
				} else {
					Some(arg)
				});
				output.split("\n").into_iter().for_each(|line| {
					if line.starts_with("Available commands") {
						println!("{}", line);
					} else {
						let mut line_split = line.split(" ");
						println!(
							"{} {}",
							line_split
								.next()
								.expect("expecting multiple elements")
								.bright_yellow(),
							line_split.into_iter().collect::<Vec<_>>().join(" ")
						);
					}
				});
			},
		));
		// exit
		r.commands
			.push(Command::new("exit", CmdArgs::None, "Exit repl", |args| {
				args.repl.exit_loop = true
			}));
		// cancel
		r.commands.push(Command::new(
			"cancel",
			CmdArgs::None,
			"Cancels more input",
			|_| (),
		));
		// cancel (with c)
		r.commands.push(Command::new(
			"c",
			CmdArgs::None,
			"Cancels more input",
			|_| (),
		));
		// load
		r.commands.push(Command::new(
			"load",
			CmdArgs::Filename,
			"load *.rs or *.rscript as inputs",
			|args| match load_and_parse(&args.arg) {
				InputResult::Program(input) => {
					debug!("loaded file: {:?}", input);
					args.repl.handle_input(input).is_ok(); // ignore result, will already be printed
				}
				InputResult::InputError(e) => println!("{}", e),
				_ => println!("haven't handled file input"),
			},
		));
		r
	}

	/// Runs the file and returns a new REPL instance.
	pub fn with_file(filename: &str) -> Self {
		let mut repl = Repl::new();
		match load_and_parse(&filename) {
			InputResult::Program(input) => {
				debug!("loaded file: {:?}", input);
				repl.handle_input(input).is_ok(); // ignore result, will already be printed
			}
			InputResult::InputError(e) => println!("{}", e),
			_ => println!("haven't handled file input"),
		}
		repl
	}

	/// Run the REPL interactively.
	///
	/// # Panics
	/// - Failure to initialise `InputReader`.
	pub fn run(mut self) {
		{
			print!("{}", "Checking for later version...".bright_yellow());
			io::stdout().flush().is_ok();
			let print_line = match query() {
				Ok(status) => match status {
					Status::UpToDate(ver) => format!(
						"{}{}",
						"Running the latest papyrus version ".bright_green(),
						ver.bright_green()
					),
					Status::OutOfDate(ver) => format!(
						"{}{}{}{}",
						"The current papyrus version ".bright_red(),
						env!("CARGO_PKG_VERSION").bright_red(),
						" is old, please update to ".bright_red(),
						ver.bright_red()
					),
				},
				Err(_) => "Failed to query crates.io".to_string(),
			};
			overwrite_current_console_line(&print_line);
			println!("",);
		} // version information.

		let mut input_rdr = InputReader::new(self.name).expect("failed to start input reader");
		let mut more = false;
		self.exit_loop = false;
		while !self.exit_loop {
			let prompt = if more {
				format!("{}.> ", self.name.color(self.prompt_colour))
			} else {
				format!("{}=> ", self.name.color(self.prompt_colour))
			};

			let res = input_rdr.read_input(&prompt);

			match res {
				InputResult::Command(name, args) => {
					debug!("read command: {} {:?}", name, args);
					more = false;
					match self.commands.find_command(&name) {
						Err(e) => println!("{}", e),
						Ok(cmd) => (cmd.action)(CommandActionArgs {
							repl: &mut self,
							arg: &args,
						}),
					};
				}
				InputResult::Program(input) => {
					debug!("read program: {:?}", input);
					more = false;
					self.handle_input(input).is_ok(); // ignore result, will already be printed
				}
				InputResult::Empty => (),
				InputResult::More => {
					more = true;
				}
				InputResult::Eof => break,
				InputResult::InputError(err) => {
					println!("{}", err);
					more = false;
				}
			};
		}
	}

	// TODO make this clean the repl as well.
	pub fn clean(&self) {
		match compile_dir().canonicalize() {
			Ok(d) => {
				let target_dir = format!("{}/target", d.to_string_lossy());
				fs::remove_dir_all(target_dir).is_ok();
			}
			_ => (),
		}
	}

	/// Runs a single program input.
	fn handle_input(&mut self, input: Input) -> Result<String, String> {
		let additionals = build_additionals(input, self.statements.len());
		let src = self.build_source(additionals.clone());
		match eval(&compile_dir(), src, self.print) {
			Ok(s) => {
				//Successful compile/runtime means we can add the new items to every program
				// crates
				additionals
					.crates
					.into_iter()
					.for_each(|c| self.crates.push(c));

				// items
				if let Some(items) = additionals.items {
					self.items.push(items);
				}

				// statements
				let mut yes = false;
				if let Some(stmts) = additionals.stmts {
					self.statements.push(stmts.stmts);
					yes = true;
				}
				if yes && self.print {
					let out_stmt = format!("[out{}]", self.statements.len() - 1);
					println!(
						"{} {}: {}",
						self.name.color(self.prompt_colour),
						out_stmt.color(self.out_colour),
						s
					);
				}
				Ok(s)
			}
			Err(s) => {
				print!("{}", s);
				io::stdout().flush().expect("flushing stdout failed");
				Err(s)
			}
		}
	}

	fn build_source(&mut self, additional: Additional) -> SourceFile {
		let mut items = self
			.items
			.iter()
			.flatten()
			.map(|x| x.to_owned())
			.collect::<Vec<_>>()
			.join("\n");
		let mut statements = self
			.statements
			.iter()
			.flatten()
			.map(|x| x.to_owned())
			.collect::<Vec<_>>()
			.join("\n");
		let crates = self
			.crates
			.iter()
			.chain(additional.crates.iter())
			.map(|x| x.clone())
			.collect();
		if let Some(i) = additional.items {
			items.push_str("\n");
			items.push_str(&i.join("\n"));
		}
		if let Some(stmts) = additional.stmts {
			statements.push('\n');
			statements.push_str(&stmts.stmts.join("\n"));
			statements.push('\n');
			statements.push_str(&stmts.print_stmt);
		}

		SourceFile {
			src: code(&statements, &items),
			file_name: String::from("mem-code"),
			file_type: SourceFileType::Rs,
			crates: crates,
		}
	}
}

/// Evaluates the source file by compiling and running the given source file.
/// Returns the stderr if unsuccessful compilation or runtime, or the evaluation print value.
/// Stderr is piped to the current stdout for compilation, with each line overwriting itself.
fn eval<P: AsRef<Path>>(
	compile_dir: &P,
	source: SourceFile,
	print: bool,
) -> Result<String, String> {
	let mut c = Exe::compile(&source, compile_dir).unwrap();

	let compilation_stderr = {
		// output stderr stream line by line, erasing each line as you go.
		let rdr = BufReader::new(c.stderr());
		let mut s = String::new();
		for line in rdr.lines() {
			let line = line.unwrap();
			if print {
				overwrite_current_console_line(&line);
			}
			s.push_str(&line);
			s.push('\n');
		}
		if print {
			overwrite_current_console_line("");
		}
		s
	};

	match c.wait() {
		Ok(exe) => {
			let mut c = exe.run(&::std::env::current_dir().unwrap());
			// print out the stdout as each line comes
			// split out on the split pattern, and do not print that section!
			let print = {
				let mut rdr = BufReader::new(c.stdout());
				let mut s = String::new();
				for line in rdr.lines() {
					let line = line.unwrap();
					let mut split = line.split(PAPYRUS_SPLIT_PATTERN);
					if let Some(first) = split.next() {
						if !first.is_empty() && print {
							println!("{}", first);
						}
					}
					if let Some(second) = split.next() {
						s.push_str(second);
					}
				}
				s
			};

			let stderr = {
				let mut s = String::new();
				let mut rdr = BufReader::new(c.stderr());
				rdr.read_to_string(&mut s).unwrap();
				s
			};

			if c.wait().success() {
				Ok(print)
			} else {
				Err(stderr)
			}
		}
		Err(_) => Err(compilation_stderr),
	}
}

fn build_additionals(input: Input, statement_num: usize) -> Additional {
	let mut additional_items = None;
	let mut additional_statements = None;
	let mut print_stmt = String::new();
	let Input {
		items,
		mut stmts,
		crates,
	} = input;

	if items.len() > 0 {
		additional_items = Some(items);
	}
	if stmts.len() > 0 {
		if let Some(mut last) = stmts.pop() {
			let expr = if !last.semi {
				print_stmt = format!(
					"println!(\"{}{{:?}}\", out{});",
					PAPYRUS_SPLIT_PATTERN, statement_num
				);
				format!("let out{} = {};", statement_num, last.expr)
			} else {
				last.expr.to_string()
			};
			last.expr = expr;
			stmts.push(last);
		}
		let stmts = stmts
			.into_iter()
			.map(|mut x| {
				if x.semi {
					x.expr.push(';');
				}
				x.expr
			})
			.collect();
		additional_statements = Some(AdditionalStatements { stmts, print_stmt });
	}

	Additional {
		items: additional_items,
		stmts: additional_statements,
		crates: crates,
	}
}

fn load_and_parse<P: AsRef<Path>>(file_path: &P) -> InputResult {
	match SourceFile::load(file_path) {
		Ok(src) => {
			// add crates back in....
			let src = format!(
				"{}\n{}",
				src.crates.into_iter().fold(String::new(), |mut acc, x| {
					acc.push_str(&x.src_line);
					acc.push('\n');
					acc
				}),
				src.src
			);
			let r = input::parse_program(&src);
			if r == InputResult::More {
				// there is a trailing a semi colon, parse with an empty fn
				debug!("parsing again as there was no returning expression");
				input::parse_program(&format!("{}\n()", src))
			} else {
				r
			}
		}
		Err(e) => InputResult::InputError(e),
	}
}

fn compile_dir() -> PathBuf {
	let dir = dirs::home_dir().unwrap_or(PathBuf::new());
	let dir = PathBuf::from(format!("{}/.papyrus", dir.to_string_lossy()));
	dir
}

fn overwrite_current_console_line(line: &str) {
	if cfg!(test) {
		println!("{}", line);
	} else {
		let (col, row) = term_cursor::get_pos().expect("getting cursor position failed");
		term_cursor::set_pos(0, row).expect("setting cursor position failed");
		for _ in 0..col {
			print!(" ");
		}
		term_cursor::set_pos(0, row).expect("setting cursor position failed");
		print!("{}", line);
		std::io::stdout().flush().expect("flushing stdout failed");
	}
}

fn code(statements: &str, items: &str) -> String {
	format!(
		r#"fn main() {{
    {stmts}
}}

{items}
"#,
		stmts = statements,
		items = items
	)
}
