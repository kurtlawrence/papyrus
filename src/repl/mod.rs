use super::compile::*;
use super::file::SourceFile;
use super::input::{self, Input, InputReader, InputResult};
use super::*;
use colored::*;
use std::io::Read as IoRead;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use term_cursor;

mod command;
mod state;

use self::command::Commands;
pub use self::command::{CmdArgs, Command, CommandActionArgs};

/// A REPL instance.
pub struct Data {
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

struct InnerData {}

pub struct Read;
pub struct Evaluate {
	result: InputResult,
}
pub struct ManualPrint;
pub struct Print {
	to_print: String,
}

pub struct Repl<S> {
	state: S,
	inner: InnerData,
	pub data: Data,
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

fn load_and_parse<P: AsRef<Path>>(file_path: P) -> InputResult {
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
