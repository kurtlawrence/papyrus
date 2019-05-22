#[macro_use]
extern crate papyrus;

use azul::prelude::*;
use papyrus_pad::pad::*;
use std::sync::{Arc, RwLock};
use papyrus::prelude::MemoryTerminal;

type TypedPadState = PadState<MyApp, String>;

struct MyApp {
    repl_term: AppValue<TypedPadState>,
}

impl std::borrow::BorrowMut<AppValue<TypedPadState>> for MyApp {
    fn borrow_mut(&mut self) -> &mut AppValue<TypedPadState> {
        &mut self.repl_term
    }
}

impl std::borrow::Borrow<AppValue<TypedPadState>> for MyApp {
    fn borrow(&self) -> &AppValue<TypedPadState> {
        &self.repl_term
    }
}

impl Layout for MyApp {
    fn layout(&self, info: LayoutInfo<Self>) -> Dom<Self> {
        Dom::div().with_child(ReplTerminal::dom(&self.repl_term, info.window))
    }
}

fn main() {
    let term = MemoryTerminal::new();

    let repl = repl_with_term!(term.clone(), String);

    let mut app = App::new(
        MyApp {
            repl_term: AppValue::new(PadState::new(
                repl,
                Arc::new(RwLock::new(12345.to_string())),
            )),
        },
        AppConfig {
            enable_logging: Some(LevelFilter::Error),
            log_file_path: Some("debug.log".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    let window = if cfg!(debug_assertions) {
        // app.create_hot_reload_window(
        //     WindowCreateOptions::default(),
        //     css::hot_reload_override_native(
        //         "styles/test.css",
        //         std::time::Duration::from_millis(1000),
        //     ),
        // )
        // .unwrap()

        app.create_window(
            WindowCreateOptions::default(),
            css::override_native(&std::fs::read_to_string("styles/test.css").unwrap()).unwrap(),
        )
        .unwrap()
    } else {
        app.create_window(WindowCreateOptions::default(), css::native())
            .unwrap()
    };

    app.run(window).unwrap();
}
