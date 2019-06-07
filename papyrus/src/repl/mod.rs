//! The repl takes the commands given and evaluates them, setting a local variable such that the data can be continually referenced. To construct a repl instance, use the macros [`repl!`](../macros.html) or [`repl_with_term!`](../macros.html).
//!
//! Repls are state machines, consisting of a read, evaluate, and print states. Each state will lead directly to the next with relevant methods. Generally a user will only use the `.read` and `.eval` methods. Calling `.run` will consume the repl and block the thread until it exits.
//!
//! You can replicate the `run` behaviour with a basic implementation:
//!
//! ```rust, ignore
//! #[macro_use]
//! extern crate papyrus;
//!
//! use papyrus::prelude::*;
//!
//! let mut repl = repl!();
//!
//! loop {
//!   let result = repl.read().eval(&mut ());
//!   match result.signal {
//!     Signal::None => (),
//!     Signal::Exit => break,
//!   }
//!   repl = result.repl.print();
//! }
//! ```
//!
//! There is also the ability to pass data around [(see _linking_)](../linking.html) and run things asynchronously. Take a look at the [github examples](https://github.com/kurtlawrence/papyrus/tree/master/examples) for more implementations and uses of the repl.
//!
//! ## Commands
//! ---
//!
//! The repl can also pass commands (which are stored in a [cmdtree](https://github.com/kurtlawrence/cmdtree)). Commands are always prefixed by a `.`. Type `.help` for information on commands.
//!
//! ### `.mut` command
//!
//! A noteworthy command is the `.mut`, which will set the next block able to mutate `app_data`. Mutating is single use, after evaluation is complete, it will not be run again. Any local variables assigned will also not be available. There are examples on github for mutating.
//!
//! ## REPL process
//! ---
//!
//! Example interaction:
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
mod any_state;
mod cmds;
mod data;
mod eval;
mod print;
mod read;

pub use cmds::*;
pub use cmdtree::Builder as CommandBuilder;

use crate::{
    input::InputResult,
    output::{self, Output},
    pfh::{self, linking::LinkingConfiguration},
};
use cmdtree::*;
use colored::*;
use crossbeam_channel::Receiver;
use std::{
    borrow::Cow,
    fmt, fs, io,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

/// The repl structure. Stored as a state machine.
/// See the [module level documentation] for more information.
///
/// A repl has different available methods depending on its state.
pub struct Repl<S, Data> {
    /// The inner repl configuration data.
    pub data: ReplData<Data>,

    state: S,

    /// A persistent flag for the prompt to change for more input.
    more: bool,

    data_mrker: PhantomData<Data>,
}

/// The inner configuration data of the repl.
pub struct ReplData<Data> {
    /// The REPL commands as a `cmdtree::Commander`.
    pub cmdtree: Commander<'static, CommandResult<Data>>,

    /// The file map of relative paths.
    file_map: pfh::FileMap,
    /// The current editing and executing file.
    current_file: PathBuf,

    /// The colour of the prompt region. ie `papyrus`.
    pub prompt_colour: Color,
    /// The colour of the out component. ie `[out0]`.
    pub out_colour: Color,

    /// The directory for which compilation is done within.
    /// Defaults to `$HOME/.papyrus/`.
    compilation_dir: PathBuf,

    /// The external crate linking configuration,
    linking: LinkingConfiguration,

    /// Flag if output is to be redirected. Generally redirection is needed,
    /// `DefaultTerminal` however will not require it (fucks linux).
    redirect_on_execution: bool,
}

/// Repl read state.
#[derive(Debug)]
pub struct Read {
    output: Output<output::Read>,
}

/// Repl evaluate state.
#[derive(Debug)]
pub struct Evaluate {
    output: Output<output::Write>,
    result: InputResult,
}

/// Repl evaluating state. This can be constructed via a `eval_async` call.
pub struct Evaluating<D> {
    jh: Receiver<EvalResult<D>>,
}

/// Repl print state.
#[derive(Debug)]
pub struct Print {
    output: Output<output::Write>,
    to_print: Cow<'static, str>,
    /// Specifies whether to print the `[out#]`
    as_out: bool,
}

/// Represents an evaluating result. Signal should be checked and handled.
pub struct EvalResult<D> {
    /// The repl, in print ready state.
    pub repl: Repl<Print, D>,
    /// The signal, if any.
    pub signal: Signal,
}

/// Return signals from evaluating.
/// Sometimes there are extra signals that result from evaluating,
/// such as the signal to exit the repl. These signals are enumerated here.
#[derive(Debug)]
pub enum Signal {
    /// No signal was sent.
    None,
    /// A signal to exit the repl has been sent.
    Exit,
}

/// The resulting state after pushing some input into the repl.
/// Take a look at the [github examples](https://github.com/kurtlawrence/papyrus/tree/master/examples) for pushing input.
pub enum PushResult<D> {
    /// The repl is still in a read state.
    Read(Repl<Read, D>),
    /// The repl is in an eval state.
    Eval(Repl<Evaluate, D>),
}

/// Result of [`read`]ing the current input buffer.
///
/// [`read`]: Repl::read
pub enum ReadResult<D> {
    /// The repl is still in a read state.
    Read(Repl<Read, D>),
    /// The repl is in an eval state.
    Eval(Repl<Evaluate, D>),
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
