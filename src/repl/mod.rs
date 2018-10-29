use super::compile;
use super::file::SourceFile;
use super::input::{self, Input, InputReader, InputResult};
use super::*;
use colored::*;
use std::path::{Path, PathBuf};

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
	pub items: Vec<Vec<String>>,
	/// Blocks of statements applied in order.
	pub statements: Vec<Vec<String>>,
	/// Flag whether to keep looping,
	exit_loop: bool,
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
}

#[derive(Clone)]
struct AdditionalStatements {
	stmts: Vec<String>,
	print_stmt: String,
}

struct EvalOut {
	captured_out: String,
	/// The `println!` out of the evaluated expression. ie Value of out#.
	eval: String,
}

impl Repl {
	/// A new REPL instance.
	pub fn new() -> Self {
		let mut r = Repl {
			commands: Vec::new(),
			items: Vec::new(),
			statements: Vec::new(),
			exit_loop: false,
			name: "papyrus",
			prompt_colour: Color::Cyan,
			out_colour: Color::BrightGreen,
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
			|repl, arg| match load_and_parse(&arg) {
				InputResult::Program(input) => {
					debug!("loaded file: {:?}", input);
					repl.handle_input(input, repl.name);
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
				repl.handle_input(input, repl.name);
			}
			InputResult::InputError(e) => println!("{}", e),
			_ => println!("haven't handled file input"),
		}
		repl
	}

	/// A new REPL instance with the given prompt.
	///
	/// # Panics
	/// - `prompt` is empty.
	pub fn with_prompt(prompt: &'static str) -> Self {
		assert!(!prompt.is_empty());
		let mut r = Repl::new();
		r.name = prompt;
		r
	}

	/// Run the REPL interactively.
	///
	/// # Panics
	/// - Failure to initialise `InputReader`.
	pub fn run(mut self) {
		let mut input = InputReader::new(self.name).expect("failed to start input reader");
		let mut more = false;
		self.exit_loop = false;
		while !self.exit_loop {
			let prompt = if more {
				format!("{}.> ", self.name.color(self.prompt_colour))
			} else {
				format!("{}=> ", self.name.color(self.prompt_colour))
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
					self.handle_input(input, self.name);
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

	/// Runs a single program input.
	fn handle_input(&mut self, input: Input, prompt: &str) {
		let additionals = build_additionals(input, self.statements.len());
		let src = self.build_source(additionals.clone());
		match self.eval(&compile_dir(), src) {
			Ok(s) => {
				//Successful compile means we can add the new items to every program
				if let Some(items) = additionals.items {
					self.items.push(items);
				}
				let mut yes = false;
				if let Some(stmts) = additionals.stmts {
					self.statements.push(stmts.stmts);
					yes = true;
				}
				if yes {
					let out_stmt = format!("[out{}]", self.statements.len() - 1);
					if !s.captured_out.is_empty() {
						println!("{}", s.captured_out);
					}
					print!(
						"{} {}: {}",
						prompt.color(self.prompt_colour),
						out_stmt.color(self.out_colour),
						s.eval
					);
				}
			}
			Err(s) => print!("{}", s),
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

		let code = format!(
			r#"pub fn main() {{
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

		SourceFile {
			src: code,
			file_name: String::from("mem-code"),
			file_type: SourceFileType::Rs,
			crates: Vec::new(),
		}
	}

	fn eval<P: AsRef<Path>>(
		&mut self,
		compile_dir: &P,
		source: SourceFile,
	) -> Result<EvalOut, String> {
		match compile::compile_and_run(&source, compile_dir, &::std::env::current_dir().unwrap()) {
			Ok(output) => {
				if output.status.success() {
					let stdout = String::from_utf8_lossy(&output.stdout);
					// parse to get the out value
					let mut split = stdout.split(PAPYRUS_SPLIT_PATTERN);
					Ok(EvalOut {
						captured_out: split
							.next()
							.expect("failed splitting string")
							.trim()
							.to_string(),
						eval: split.next().unwrap_or("").to_string(),
					})
				} else {
					Err(String::from_utf8_lossy(&output.stderr).to_string())
				}
			}
			Err(e) => Err(format!("{}", e)),
		}
	}
}

fn build_additionals(input: Input, statement_num: usize) -> Additional {
	let mut additional_items = None;
	let mut additional_statements = None;
	let mut print_stmt = String::new();
	let Input { items, mut stmts } = input;

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
	}
}

fn load_and_parse<P: AsRef<Path>>(file_path: &P) -> InputResult {
	match SourceFile::load(file_path) {
		Ok(src) => {
			let r = input::parse_program(&src.src);
			if r == InputResult::More {
				// there is a trailing a semi colon, parse with an empty fn
				debug!("parsing again as there was no returning expression");
				input::parse_program(&format!("{}\n()", src.src))
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn load_rs_source() {
		let mut repl = Repl::new();
		for src_file in RS_FILES.iter() {
			let file = format!("test-src/{}", src_file);
			println!("{}", file);
			let res = load_and_parse(&file);
			match res {
				InputResult::Program(input) => {
					let additionals = build_additionals(input, repl.statements.len());
					let src = repl.build_source(additionals);
					let eval = repl.eval(&"test", src);
					let b = eval.is_ok();
					if let Err(e) = eval {
						println!("{}", e);
					}
					assert!(b);
				}
				InputResult::InputError(e) => {
					println!("{}", e);
					panic!("should have parsed as program, got input error")
				}
				InputResult::More => panic!("should have parsed as program, got more"),
				InputResult::Command(_, _) => panic!("should have parsed as program, got command"),
				InputResult::Empty => panic!("should have parsed as program, got empty"),
				InputResult::Eof => panic!("should have parsed as program, got Eof"),
			}
		}
	}

	#[test]
	fn load_rscript_script() {
		let mut repl = Repl::new();
		for src_file in RSCRIPT_FILES.iter() {
			let file = format!("test-src/{}", src_file);
			println!("{}", file);
			let res = load_and_parse(&file);
			match res {
				InputResult::Program(input) => {
					let additionals = build_additionals(input, repl.statements.len());
					let src = repl.build_source(additionals);
					let eval = repl.eval(&"test", src);
					let b = eval.is_ok();
					if let Err(e) = eval {
						println!("{}", e);
					}
					assert!(b);
				}
				InputResult::InputError(e) => {
					println!("{}", e);
					panic!("should have parsed as program, got input error")
				}
				InputResult::More => panic!("should have parsed as program, got more"),
				InputResult::Command(_, _) => panic!("should have parsed as program, got command"),
				InputResult::Empty => panic!("should have parsed as program, got empty"),
				InputResult::Eof => panic!("should have parsed as program, got Eof"),
			}
		}
	}
}
