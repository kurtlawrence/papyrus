#[macro_use]
extern crate papyrus;

use azul::prelude::*;
use papyrus_pad::*;
use std::sync::{Arc, RwLock};

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
    fn layout(&self, mut info: LayoutInfo<Self>) -> Dom<Self> {
        Dom::div().with_child(ReplTerminal::dom(&self.repl_term, &mut info))
    }
}

fn main() {
    let repl = repl!(String);

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

    let css = create_css();

    let window = if cfg!(debug_assertions) {
        std::fs::write("hot-reload.css", create_css()).unwrap();
        app.create_hot_reload_window(
            WindowCreateOptions::default(),
            css::hot_reload("hot-reload.css", std::time::Duration::from_millis(1000)),
        )
        .unwrap()

    // app.create_window(WindowCreateOptions::default(), css::from_str(&css).unwrap())
    //     .unwrap()
    } else {
        app.create_window(WindowCreateOptions::default(), css::from_str(&css).unwrap())
            .unwrap()
    };

    app.run(window).unwrap();
}

fn create_css() -> String {
    use azul_theming::*;

    let mut css = String::new();

    css.push_str(papyrus_pad::PAD_CSS);

    inject_theme(&css, &themes::dark_theme())
}
