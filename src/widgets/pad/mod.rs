mod eval_state;
mod repl_terminal;
mod css;

pub use self::repl_terminal::{ReplTerminal};
pub use self::css::PAD_CSS;

use crate::prelude::*;
use azul::prelude::*;
use eval_state::EvalState;
use linefeed::memory::MemoryTerminal;
use std::sync::{Arc, Mutex};

pub struct PadState<Data> {
    repl: EvalState<Data>,
    terminal: MemoryTerminal,
    last_terminal_string: String,
    eval_daemon_id: TimerId,
    data: Arc<Mutex<Data>>,
}

impl<Data: 'static> PadState<Data> {
    pub fn new(repl: Repl<repl::Read, MemoryTerminal, Data>, data: Arc<Mutex<Data>>) -> Self {
        let term = repl.terminal().clone();
        Self {
            repl: EvalState::new(repl),
            terminal: term,
            last_terminal_string: String::new(),
            eval_daemon_id: TimerId::new(),
            data,
        }
    }
}
