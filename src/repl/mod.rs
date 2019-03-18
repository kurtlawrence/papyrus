//! The repl takes the commands given and evaluates them, setting a local variable such that the data can be continually referenced.
//!
//! ```sh
//! papyrus=> let a = 1;
//! papyrus.> a
//! papyrus [out0]: 1
//! papyrus=>
//! ```
//!
//! Here we define a variable `let a = 1;`. Papyrus knows that the end result is not an expression (given the trailing semi colon) so waits for more input (`.>`). We then give it `a` which is an expression and gets evaluated. If compilation is successful the expression is set to the variable `out0` (where the number will increment with expressions) and then be printed with the `Debug` trait. If an expression evaluates to something that is not `Debug` then you will receive a compilation error. Finally the repl awaits more input `=>`.
//!
//! > The expression is using `let out# = <expr>;` behind the scenes.
//!
//! You can also define structures and functions.
//!
//! ```sh
//! papyrus=> fn a(i: u32) -> u32 {
//! papyrus.> i + 1
//! papyrus.> }
//! papyrus=> a(1)
//! papyrus [out0]: 2
//! papyrus=>
//! ```
//!
//! ```txt
//! papyrus=> #[derive(Debug)] struct A {
//! papyrus.> a: u32,
//! papyrus.> b: u32
//! papyrus.> }
//! papyrus=> let a = A {a: 1, b: 2};
//! papyrus.> a
//! papyrus [out0]: A { a: 1, b: 2 }
//! papyrus=>
//! ```
//!
//! Please help if the Repl cannot parse your statements, or help with documentation! [https://github.com/kurtlawrence/papyrus](https://github.com/kurtlawrence/papyrus).
mod command;
mod eval;
mod print;
mod read;
mod writer;

use self::command::Commands;
use cmdtree::*;
use colored::*;
use crossbeam::channel::Receiver;
use crossbeam::thread::ScopedJoinHandle;
use input::{InputReader, InputResult};
use linefeed::terminal::Terminal;
use pfh::{linking::LinkingConfiguration, SourceFile};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use self::command::{CmdArgs, Command};

pub struct ReplData<Data> {
	/// The REPL commands as a `cmdtree::Commander`.
	pub cmdtree: Commander<'static, CommandResult>,
	/// The file map of relative paths.
	pub file_map: HashMap<PathBuf, SourceFile>,
	/// The current editing and executing file.
	pub current_file: PathBuf,
	/// App and prompt text.
	pub name: &'static str,
	/// The colour of the prompt region. ie `papyrus`.
	pub prompt_colour: Color,
	/// The colour of the out component. ie `[out0]`.
	pub out_colour: Color,
	/// The directory for which compilation is done within.
	/// Defaults to `$HOME/.papyrus/`.
	pub compilation_dir: PathBuf,
	/// The external crate linking configuration,
	linking: LinkingConfiguration,
	data_mrker: PhantomData<Data>,
	/// Flag if output is to be redirected. Generally redirection is needed, `DefaultTerminal` however will not require it (fucks linux).
	redirect_on_execution: bool,
}

struct ReplTerminal<Term: Terminal> {
	/// The underlying terminal of `input_rdr`, used to directly control terminal
	/// Kept as a `Arc` such that multiple references to the terminal can be shared across threads.
	/// Lucky for us that `Terminal` implements an atomic locking interface.
	terminal: Arc<Term>,
	/// The persistent input reader.
	input_rdr: InputReader<Term>,
}

struct Writer<'a, T: Terminal>(&'a T);
/// This is done to be able to `Send` a writer with a reference to the terminal in it.
struct OwnedWriter<T: Terminal>(Arc<T>);

pub struct Read;
pub struct Evaluate {
	result: InputResult,
}
pub struct Evaluating<'data, Term: Terminal, Data> {
	jh: Receiver<Result<Repl<'data, Print, Term, Data>, EvalSignal>>,
}
pub struct ManualPrint;
pub struct Print {
	to_print: String,
	/// Specifies whether to print the `[out#]`
	as_out: bool,
}

pub struct Repl<'data, S, Term: Terminal, Data> {
	state: S,
	terminal: ReplTerminal<Term>,
	pub data: &'data mut ReplData<Data>,
}

pub enum CommandResult {
	CancelInput,
}

#[derive(Debug)]
pub enum EvalSignal {
	Exit,
}

pub enum PushResult<'a, Term: Terminal, Data> {
	Read(Repl<'a, Read, Term, Data>),
	Eval(Repl<'a, Evaluate, Term, Data>),
}

impl<Data> Default for ReplData<Data> {
	fn default() -> Self {
		let lib = SourceFile::lib();
		let lib_path = lib.path.clone();
		let mut map = HashMap::new();
		map.insert(lib_path.clone(), lib);

		let mut r = ReplData {
			cmdtree: Builder::new("papyrus")
				.into_commander()
				.expect("empty should pass"),
			file_map: map,
			current_file: lib_path,
			name: "papyrus",
			prompt_colour: Color::Cyan,
			out_colour: Color::BrightGreen,
			compilation_dir: default_compile_dir(),
			linking: LinkingConfiguration::default(),
			data_mrker: PhantomData,
			redirect_on_execution: true,
		};

		r.with_cmdtree_builder(Builder::new("papyrus"))
			.expect("should build fine")
	}
}

impl<Data> ReplData<Data> {
	pub fn with_compilation_dir<P: AsRef<Path>>(mut self, dir: P) -> io::Result<Self> {
		let dir = dir.as_ref();
		if !dir.exists() {
			fs::create_dir_all(dir)?;
		}
		assert!(dir.is_dir());
		self.compilation_dir = dir.to_path_buf();
		Ok(self)
	}

	/// Uses the given `Builder` as the root of the command tree.
	/// The builder is amended with the `esc` command at the root, an error will be
	/// returned if the command already exists.
	pub fn with_cmdtree_builder(
		mut self,
		builder: Builder<'static, CommandResult>,
	) -> Result<Self, BuildError> {
		let cmdr = builder
			.root()
			.add_action("esc", "Cancels more input", |_| CommandResult::CancelInput)
			.into_commander()?;

		self.cmdtree = cmdr;
		Ok(self)
	}

	/// Specify that the repl will link an external crate reference.
	/// Overwrites previously specified crate name.
	/// Uses `ReplData.compilation_dir` to copy `rlib` file into.
	///
	/// [See documentation](https://kurtlawrence.github.io/papyrus/repl/linking.html)
	pub fn with_extern_crate(
		mut self,
		crate_name: &'static str,
		rlib_path: Option<&str>,
	) -> io::Result<Self> {
		self.linking =
			self.linking
				.link_external_crate(&self.compilation_dir, crate_name, rlib_path)?;
		Ok(self)
	}

	/// Not meant to used by developer. Use the macros instead.
	/// [See _linking_ module](../pfh/linking.html)
	pub fn set_data_type(mut self, data_type: &str) -> Self {
		self.linking = self.linking.with_data(data_type);
		self
	}

	pub fn linking(&self) -> &LinkingConfiguration {
		&self.linking
	}
}

/// `$HOME/.papyrus`
fn default_compile_dir() -> PathBuf {
	dirs::home_dir().unwrap_or(PathBuf::new()).join(".papyrus/")
}

#[test]
fn test_default_compile_dir() {
	let dir = default_compile_dir();
	println!("{}", dir.display());
	assert!(dir.ends_with(".papyrus/"));
	if cfg!(windows) {
		assert!(dir.starts_with("C:\\Users\\"));
	} else {
		assert!(dir.starts_with("/home/"));
	}
}
