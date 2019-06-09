use azul::prelude::*;

// macro_rules! cb {
// 	// TODO Work out how to append ident with something to make priv_$fn
// 	// ($fn:ident) => {
//     //     fn $fn(
//     //         data: &StackCheckedPointer<T>,
//     //         app_state_no_data: &mut AppStateNoData<T>,
//     //         window_event: &mut CallbackInfo<T>,
//     //     ) -> UpdateScreen {
//     //         data.invoke_mut(Self::$fn, app_state_no_data, window_event)
//     //     }
//     // };
// }

pub mod ansi_renderer;
pub mod colour;
mod completion;
mod css;
mod eval_state;
pub mod pad;
pub mod repl_terminal;

pub use self::css::PAD_CSS;
pub use self::repl_terminal::ReplTerminal;

use eval_state::EvalState;
use std::sync::{Arc, RwLock};

pub struct PadState<T, Data> {
    // Common repl
    repl: EvalState<Data>,
    data: Arc<RwLock<Data>>,

    // ReplTerminal
    input_buffer: String,

    term_render: ansi_renderer::ReplOutputRenderer,

    after_eval_fn: fn(&mut T, &mut AppResources),
    eval_timer_id: TimerId,

    completion: completion::CompletionPromptState,
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
