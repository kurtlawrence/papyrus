mod css;
mod eval_state;
mod prompt;
mod repl_terminal;

pub use self::css::PAD_CSS;
pub use self::repl_terminal::{add_terminal_text, create_terminal_string, ReplTerminal};

use azul::prelude::*;
use eval_state::EvalState;
use papyrus::complete;
use papyrus::prelude::*;
use std::sync::{Arc, RwLock};

pub struct PadState<T, Data> {
    repl: EvalState<Data>,
    terminal: MemoryTerminal,
    last_terminal_string: String,
    eval_daemon_id: TimerId,
    data: Arc<RwLock<Data>>,
    after_eval_fn: fn(&mut T, &mut AppResources),
    completers: Completers,
}

impl<T, D> PadState<T, D> {
    pub fn new(repl: Repl<repl::Read, MemoryTerminal, D>, data: Arc<RwLock<D>>) -> Self {
        let term = repl.terminal().clone();

        let completers = build_completer(&repl.data);

        Self {
            repl: EvalState::new(repl),
            terminal: term,
            last_terminal_string: String::new(),
            eval_daemon_id: TimerId::new(),
            data,
            after_eval_fn: none,
            completers,
        }
    }

    pub fn with_after_eval_fn(mut self, func: fn(&mut T, &mut AppResources)) -> Self {
        self.after_eval_fn = func;
        self
    }

    /// Functions to run after the evaluation phase finished.
    fn eval_finished(&mut self) {
        if let Some(repl) = self.repl.brw_repl() {
            self.completers = build_completer(&repl.data);
        }
    }
}

pub struct Completers {
    cmds_tree: complete::cmdr::TreeCompleter,
    mods: complete::modules::ModulesCompleter,
    code: complete::code::CodeCompleter,
}

fn none<T>(_: &mut T, _: &mut AppResources) {}

fn build_completer<D>(repl_data: &ReplData<D>) -> Completers {
    let cmds_tree = complete::cmdr::TreeCompleter::build(&repl_data.cmdtree);

    let mods =
        complete::modules::ModulesCompleter::build(&repl_data.cmdtree, &repl_data.file_map());

    let code = complete::code::CodeCompleter::build(repl_data);

    Completers {
        cmds_tree,
        mods,
        code,
    }
}
