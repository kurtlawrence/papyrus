//! The repl takes the commands given and evaluates them, setting a local variable such that the data can be continually referenced. To construct a repl instance, use the macros [`repl!`].
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
mod data;
mod eval;
mod print;
mod read;

use crate::{
    cmds::CommandResult,
    code::ModsMap,
    input::InputResult,
    linking::{self, LinkingConfiguration},
    output::{self, Output},
};
use cmdtree::*;
use colored::*;
use crossbeam_channel::Receiver;
use kserd::Kserd;
use std::{
    borrow::Cow,
    fmt, fs, io,
    marker::PhantomData,
    path::{Path, PathBuf},
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
    pub cmdtree: Commander<CommandResult<Data>>,

    /// The modules map of relative paths.
    pub(crate) mods_map: ModsMap,
    /// The current editing and executing mod.
    pub(crate) current_mod: PathBuf,

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

    /// Flag for editing a statement, item, or crate.
    ///
    /// If a value is set when an evaluation starts, the input buffer
    /// will be used to overwrite the element at the given index (if it exists).
    /// Compilation and evaluation could both fail, but the change _will not be reverted_.
    ///
    /// If the index is outside the array bounds then there will be no change. Evaluation
    /// phase will still run.
    ///
    /// [`read()`]: Repl::read
    pub editing: Option<EditingIndex>,
    /// The rust source code as a string which is being edited.
    ///
    /// This is helpful if an alteration has been requested and you want to
    /// show the old source code. It is recommended to `.take()` the value
    /// to avoid repeating the contents.
    pub editing_src: Option<String>,
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
    data: EvalOutput,
}

/// Was the eval something that produces data??
#[derive(Debug)]
enum EvalOutput {
    /// If there is data, then it should be prefixed with `[out#]`.
    Data(Kserd<'static>),
    Print(Cow<'static, str>),
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
    /// Signal to run the evaluation loop again with the inner
    /// value as the line input.
    ///
    /// This is usually signaled when [`EditReplace`] is instigated.
    /// Re-evaulation is signalled rather than handled as the input
    /// may be not enough to complete a full repl cycle.
    ///
    /// [`EditReplace`]: super::cmds::CommandResult
    ReEvaluate(String),
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

/// The index of the statement group, item, or crate being edited.
#[derive(Copy, Clone, Debug)]
pub struct EditingIndex {
    /// Type being edited.
    pub editing: Editing,
    /// Index.
    pub index: usize,
}

/// Type being edited
#[derive(Copy, Clone, Debug)]
pub enum Editing {
    /// A statement group, corresponding to the `out#`.
    Stmt,
    /// An item, such as fn or struct.
    Item,
    /// A crate.
    Crate,
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
