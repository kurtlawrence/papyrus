mod eval_state;
mod repl_terminal;

pub use self::repl_terminal::{ReplTerminal, PAD_CSS};

use crate::prelude::*;
use azul::prelude::*;
use eval_state::EvalState;
use linefeed::memory::MemoryTerminal;
use std::sync::{Arc, Mutex};

pub struct PadState<'a, Data, Ref> {
    repl: EvalState<Data, Ref>,
    terminal: MemoryTerminal,
    last_terminal_string: String,
    eval_daemon_id: DaemonId,
    data: Box<RefCast<Data> + 'a>,
}

trait RefCast<D> {
    fn as_noref(&self) -> D;
    fn as_brw(&self) -> &Arc<D>;
    fn as_brw_mut(&mut self) -> &Arc<Mutex<D>>;
}

impl<'a, Data: 'a + Clone> PadState<'a, Data, linking::NoRef> {
    pub fn new(repl: Repl<repl::Read, MemoryTerminal, Data, linking::NoRef>, data: Data) -> Self {
        let term = repl.terminal_inner().clone();
        Self {
            repl: EvalState::new(repl),
            terminal: term,
            last_terminal_string: String::new(),
            eval_daemon_id: DaemonId::new(),
            data: Box::new(NoRefDataWrapper { data }),
        }
    }
}

struct NoRefDataWrapper<D> {
    data: D,
}

impl<D: Clone> RefCast<D> for NoRefDataWrapper<D> {
    fn as_noref(&self) -> D {
        self.data.clone()
    }
    fn as_brw(&self) -> &Arc<D> {
        unreachable!("can't borrow NoRef data");
    }
    fn as_brw_mut(&mut self) -> &Arc<Mutex<D>> {
        unreachable!("can't borrow mutable NoRef data");
    }
}

struct BrwDataWrapper<D> {
    data: Arc<D>,
}

impl<D> RefCast<D> for BrwDataWrapper<D> {
    fn as_noref(&self) -> D {
        unreachable!("can't deref Brw data");
    }
    fn as_brw(&self) -> &Arc<D> {
        &self.data
    }
    fn as_brw_mut(&mut self) -> &Arc<Mutex<D>> {
        unreachable!("can't borrow mutable Brw data");
    }
}

struct BrwMutDataWrapper<D> {
    data: Arc<Mutex<D>>,
}

impl<D> RefCast<D> for BrwMutDataWrapper<D> {
    fn as_noref(&self) -> D {
        unreachable!("can't deref BrwMut data");
    }
    fn as_brw(&self) -> &Arc<D> {
        unreachable!("can't borrow BrwMut data");
    }
    fn as_brw_mut(&mut self) -> &Arc<Mutex<D>> {
        &self.data
    }
}
