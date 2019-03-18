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
mod data;
mod eval;
mod print;
mod read;
mod writer;

use crate::input::{InputReader, InputResult};
use crate::pfh::{linking::LinkingConfiguration, SourceFile};
use cmdtree::*;
use colored::*;
use crossbeam::channel::Receiver;
use crossbeam::thread::ScopedJoinHandle;
use linefeed::terminal::Terminal;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Repl<S, Term: Terminal, Data, Ref> {
	pub data: ReplData,
	state: S,
	terminal: ReplTerminal<Term>,
	data_mrker: PhantomData<Data>,
	ref_mrker: PhantomData<Ref>,
}

impl<S, T: Terminal, D, R> Repl<S, T, D, R> {
	fn move_state<N>(self, state: N) -> Repl<N, T, D, R> {
		Repl {
			state: state,
			terminal: self.terminal,
			data: self.data,
			data_mrker: self.data_mrker,
			ref_mrker: self.ref_mrker,
		}
	}
}

pub struct ReplData {
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
pub struct Evaluating<Term: Terminal, Data, Ref> {
	jh: Receiver<Result<Repl<Print, Term, Data, Ref>, EvalSignal>>,
}
pub struct Print {
	to_print: String,
	/// Specifies whether to print the `[out#]`
	as_out: bool,
}

pub enum CommandResult {
	CancelInput,
}

#[derive(Debug)]
pub enum EvalSignal {
	Exit,
}

pub enum PushResult<Term: Terminal, Data, Ref> {
	Read(Repl<Read, Term, Data, Ref>),
	Eval(Repl<Evaluate, Term, Data, Ref>),
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
