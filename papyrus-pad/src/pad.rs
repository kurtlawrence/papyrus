

use super::*;
use azul::prelude::*;
use eval_state::EvalState;
use papyrus::complete;
use papyrus::prelude::*;
use std::sync::{Arc, RwLock};



impl<T, D> PadState<T, D> {
    pub fn new(repl: Repl<repl::Read, MemoryTerminal, D>, data: Arc<RwLock<D>>) -> Self {
        let term = repl.terminal().clone();

        let completers = prompt::Completers::build(&repl.data);

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
    pub fn eval_finished(&mut self) {
        if let Some(repl) = self.repl.brw_repl() {
            self.completers = prompt::Completers::build(&repl.data);
        }
    }
}


fn none<T>(_: &mut T, _: &mut AppResources) {}


