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
mod cmds;
mod data;
mod eval;
mod print;
mod read;
mod writer;

pub use cmdtree::Builder as CommandBuilder;

use crate::complete::*;
use crate::input::{InputReader, InputResult};
use crate::pfh::{self, linking::LinkingConfiguration};
use cmdtree::*;
use colored::*;
use crossbeam::channel::Receiver;
use linefeed::terminal::Terminal;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{
    fmt, fs,
    io::{self, Write},
};

/// The repl structure. Stored as a state machine.
/// See the [module level documentation] for more information.
///
/// A repl has different available methods depending on its state.
pub struct Repl<S, Term: Terminal, Data> {
    /// The inner repl configuration data.
    pub data: ReplData<Data>,
    state: S,
    terminal: ReplTerminal<Term>,
    /// A persistent flag for the prompt to change for more input.
    more: bool,
    data_mrker: PhantomData<Data>,
}

impl<S, T: Terminal, D> Repl<S, T, D> {
    /// The terminal that the repl reads from and writes to.
    pub fn terminal(&self) -> &T {
        self.terminal.terminal.as_ref()
    }

    fn move_state<N>(self, state: N) -> Repl<N, T, D> {
        Repl {
            state: state,
            terminal: self.terminal,
            data: self.data,
            more: self.more,
            data_mrker: self.data_mrker,
        }
    }
}

impl<S, T: Terminal, D> Repl<S, T, D> {
	/// Set completion on the terminal.
    pub fn set_completion(&mut self, combined: crate::complete::CombinedCompleter<'static, T>) {
        self.terminal
            .input_rdr
            .set_completer(std::sync::Arc::new(combined));
    }
}

impl<S: fmt::Debug, T: Terminal, D> fmt::Debug for Repl<S, T, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Repl in <{:?}> state instance", self.state)
    }
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

/// Repl read state.
#[derive(Debug)]
pub struct Read;
/// Repl evaluate state.
pub struct Evaluate {
    result: InputResult,
}
/// Repl evaluating state. This can be constructed via a `eval_async` call.
pub struct Evaluating<Term: Terminal, Data> {
    jh: Receiver<EvalResult<Term, Data>>,
}
/// Repl print state.
pub struct Print {
    to_print: Cow<'static, str>,
    /// Specifies whether to print the `[out#]`
    as_out: bool,
}

/// The result of a [`cmdtree action`](https://docs.rs/cmdtree/builder/trait.BuilderChain.html#tymethod.add_action).
/// This result is handed in the repl's evaluating stage, and can alter `ReplData`.
pub enum CommandResult<Data> {
    /// Flag to begin a mutating block.
    BeginMutBlock,
    /// Take an action on the `ReplData`.
    ActionOnReplData(ReplDataAction<Data>),
    /// Take an action on `Data`.
    ActionOnAppData(AppDataAction<Data>),
    /// A blank variant with no action.
    Empty,
}

impl<D> CommandResult<D> {
    /// Convenience function boxing an action on app data.
    pub fn app_data_fn<F: 'static + for<'w> Fn(&mut D, Box<Write + 'w>) -> String>(
        func: F,
    ) -> Self {
        CommandResult::ActionOnAppData(Box::new(func))
    }

    /// Convenience function boxing an action on repl data.
    pub fn repl_data_fn<F: 'static + for<'w> Fn(&mut ReplData<D>, Box<Write + 'w>) -> String>(
        func: F,
    ) -> Self {
        CommandResult::ActionOnReplData(Box::new(func))
    }
}

/// The action to take. Passes through a mutable reference to the `ReplData`.
/// TODO could this not just be W: Write rather than box? Or rather just &Write
/// Can't be W as it would add another generic argument.
pub type ReplDataAction<D> = Box<for<'w> Fn(&mut ReplData<D>, Box<Write + 'w>) -> String>;

/// The action to take. Passes through a mutable reference to the `Data`.
pub type AppDataAction<D> = Box<for<'w> Fn(&mut D, Box<Write + 'w>) -> String>;

/// Represents an evaluating result. Signal should be checked and handled.
pub struct EvalResult<Term: Terminal, Data> {
    /// The repl, in print ready state.
    pub repl: Repl<Print, Term, Data>,
    /// The signal, if any.
    signal: Signal,
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
pub enum PushResult<Term: Terminal, Data> {
    /// The repl is still in a read state.
    Read(Repl<Read, Term, Data>),
    /// The repl is in an eval state.
    Eval(Repl<Evaluate, Term, Data>),
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
