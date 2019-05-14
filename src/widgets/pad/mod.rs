mod css;
mod eval_state;
mod repl_terminal;

pub use self::css::PAD_CSS;
pub use self::repl_terminal::{create_terminal_string, ReplTerminal};

use crate::prelude::*;
use azul::prelude::*;
use eval_state::EvalState;
use linefeed::memory::MemoryTerminal;
use std::sync::{Arc, RwLock};

pub struct PadState<T, Data> {
    repl: EvalState<Data>,
    terminal: MemoryTerminal,
    last_terminal_string: String,
    eval_daemon_id: TimerId,
    data: Arc<RwLock<Data>>,
    after_eval_fn: fn(&mut T, &mut AppResources),
}

impl<T, Data: 'static> PadState<T, Data> {
    pub fn new(repl: Repl<repl::Read, MemoryTerminal, Data>, data: Arc<RwLock<Data>>) -> Self {
        let term = repl.terminal().clone();
        Self {
            repl: EvalState::new(repl),
            terminal: term,
            last_terminal_string: String::new(),
            eval_daemon_id: TimerId::new(),
            data,
            after_eval_fn: none,
        }
    }

    pub fn with_after_eval_fn(mut self, func: fn(&mut T, &mut AppResources)) -> Self {
        self.after_eval_fn = func;
        self
    }
}

fn none<T>(_: &mut T, _: &mut AppResources) {}
