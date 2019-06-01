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
pub use self::repl_terminal::{add_terminal_text, create_terminal_string, ReplTerminal};

use eval_state::EvalState;
use papyrus::prelude::MemoryTerminal;
use std::sync::{Arc, RwLock};

pub struct PadState<T, Data> {
    repl: EvalState<Data>,
    terminal: MemoryTerminal,
    last_terminal_string: String,
    eval_daemon_id: TimerId,
    data: Arc<RwLock<Data>>,
    after_eval_fn: fn(&mut T, &mut AppResources),

    term_render: ansi_renderer::AnsiRenderer,

    completion: completion::CompletionPromptState,
}
