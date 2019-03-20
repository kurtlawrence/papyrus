mod eval_state;
mod repl_terminal;

pub use self::repl_terminal::{ReplTerminal, PAD_CSS};

use crate::prelude::*;
use azul::prelude::*;
use eval_state::EvalState;
use linefeed::memory::MemoryTerminal;

pub struct PadState {
    repl: EvalState<(), linking::NoRef>,
    terminal: MemoryTerminal,
    last_terminal_string: String,
    eval_daemon_id: DaemonId,
}

impl PadState {
    pub fn new(repl: Repl<repl::Read, MemoryTerminal, (), linking::NoRef>) -> Self {
        let term = repl.terminal_inner().clone();
        Self {
            repl: EvalState::new(repl),
            terminal: term,
            last_terminal_string: String::new(),
            eval_daemon_id: DaemonId::new(),
        }
    }
}
