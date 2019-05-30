use azul::prelude::*;

macro_rules! cb {
    ($type:ident, $fn:ident) => {
        fn $fn(
            data: &StackCheckedPointer<T>,
            app_state_no_data: &mut AppStateNoData<T>,
            window_event: &mut CallbackInfo<T>,
        ) -> UpdateScreen {
            data.invoke_mut($type::$fn, app_state_no_data, window_event)
        }
    };
}



pub mod colour;
pub mod pad;
mod css;
mod prompt;
mod repl_terminal;
mod eval_state;

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
    completers: prompt::Completers,
}


