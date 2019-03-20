mod eval_state;
mod repl_terminal;

pub use self::repl_terminal::{ReplTerminal, PAD_CSS};

use crate::prelude::*;
use azul::prelude::*;
use eval_state::EvalState;
use linefeed::memory::MemoryTerminal;
use std::sync::{Arc, Mutex};

pub struct PadState<Data> {
    repl: RefTypeVariant<Data>,
    terminal: MemoryTerminal,
    last_terminal_string: String,
    eval_daemon_id: DaemonId,
    data: Box<RefCast<Data> + 'static>,
}

enum RefTypeVariant<Data> {
    NoRef(EvalState<Data, linking::NoRef>),
    Brw(EvalState<Data, linking::Brw>),
    BrwMut(EvalState<Data, linking::BrwMut>),
}

trait RefCast<D> {
    fn as_noref(&self) -> D;
    fn as_brw(&self) -> &Arc<D>;
    fn as_brw_mut(&self) -> &Arc<Mutex<D>>;
}

impl<Data: 'static + Clone> PadState<Data> {
    pub fn new_no_ref(
        repl: Repl<repl::Read, MemoryTerminal, Data, linking::NoRef>,
        data: Data,
    ) -> Self {
        let term = repl.terminal_inner().clone();
        Self {
            repl: RefTypeVariant::NoRef(EvalState::new(repl)),
            terminal: term,
            last_terminal_string: String::new(),
            eval_daemon_id: DaemonId::new(),
            data: Box::new(NoRefDataWrapper { data }),
        }
    }
}

impl<Data: 'static> PadState<Data> {
    pub fn new_brw(
        repl: Repl<repl::Read, MemoryTerminal, Data, linking::Brw>,
        data: Arc<Data>,
    ) -> Self {
        let term = repl.terminal_inner().clone();
        Self {
            repl: RefTypeVariant::Brw(EvalState::new(repl)),
            terminal: term,
            last_terminal_string: String::new(),
            eval_daemon_id: DaemonId::new(),
            data: Box::new(BrwDataWrapper { data }),
        }
    }

    pub fn new_brw_mut(
        repl: Repl<repl::Read, MemoryTerminal, Data, linking::BrwMut>,
        data: Arc<Mutex<Data>>,
    ) -> Self {
        let term = repl.terminal_inner().clone();
        Self {
            repl: RefTypeVariant::BrwMut(EvalState::new(repl)),
            terminal: term,
            last_terminal_string: String::new(),
            eval_daemon_id: DaemonId::new(),
            data: Box::new(BrwMutDataWrapper { data }),
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
    fn as_brw_mut(&self) -> &Arc<Mutex<D>> {
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
    fn as_brw_mut(&self) -> &Arc<Mutex<D>> {
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
    fn as_brw_mut(&self) -> &Arc<Mutex<D>> {
        &self.data
    }
}
