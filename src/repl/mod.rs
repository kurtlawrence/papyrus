//! The REPL API.
//!
//! The REPL uses a state machine to control what methods can be applied to it.
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
    collections::VecDeque,
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

    /// Stored loaded libraries of the papyrus mem code.
    loadedlibs: VecDeque<libloading::Library>,
    /// Limit the number of loaded libraries that are kept in memory and not dropped.
    ///
    /// There exists a use pattern which can create segmentation faults if code defined in the
    /// library is called once the library goes out of scope and is dropped. Detailed in
    /// [#44](https://github.com/kurtlawrence/papyrus/issues/44).
    ///
    /// Libraries are returned on a successful execution and stored in a vector. Once the vector
    /// reaches the size limit, the _oldest_ library is removed and dropped, freeing resources.
    ///
    /// The default is to keep the size limit at zero, thus ensuring no libraries are kept in
    /// memory. This is recommended unless issues are arising from esoteric use cases.
    pub loaded_libs_size_limit: usize,
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
