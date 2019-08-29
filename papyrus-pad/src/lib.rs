use azul::prelude::*;

// TODO Make cargo run pretty the output, pad tree needs work
// TODO Try without TextSlice, I think it is fucking things up
// TODO Clean warnings
// TODO Celebrate??

macro_rules! cb {
    ($priv:ident, $fn:ident) => {
        fn $priv(info: DefaultCallbackInfoUnchecked<T>) -> UpdateScreen {
            unsafe { info.invoke_callback(Self::$fn) }
        }
    };
}

pub mod ansi_renderer;
pub mod colour;
mod completion;
mod eval_state;
mod history;
pub mod mods_tree;
pub mod pad;
pub mod repl_terminal;
pub mod styles;

pub use self::mods_tree::ReplModulesTree;
pub use self::repl_terminal::ReplTerminal;

use eval_state::EvalState;
use std::sync::{Arc, RwLock};

pub struct PadState<T, Data> {
    // Common repl
    repl: EvalState<Data>,
    data: Arc<RwLock<Data>>,

    // ReplTerminal
    /// This is the input buffer _line_, so completions and such need to work off this.
    input_buffer: String,
    history: history::History,

    term_render: ansi_renderer::ReplOutputRenderer,

    after_eval_fn: fn(&mut T, &mut AppResources),
    eval_timer_id: TimerId,

    completion: AppValue<completion::CompletionPromptState<T>>,
    completion_timer_id: TimerId,
}

pub fn kb_seq<T, F: FnOnce() -> T>(
    kb_state: &KeyboardState,
    keys: &[AcceleratorKey],
    result: F,
) -> Option<T> {
    if keys.iter().all(|key| key.matches(kb_state)) {
        Some(result())
    } else {
        None
    }
}
