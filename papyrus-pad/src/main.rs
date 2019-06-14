#[macro_use]
extern crate papyrus;

use azul::prelude::*;
use azul::window::{Window, WindowCreateError, WindowSize};
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
        Dom::div()
            .with_id("pad")
            .with_child(ReplModulesTree::dom(&self.repl_term, &mut info))
            .with_child(ReplTerminal::dom(&self.repl_term, &mut info))
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

    let mut window_state = WindowState::default();
    window_state.size = WindowSize {
        dimensions: LogicalSize {
            width: 1200.0,
            height: 1000.0,
        },
        ..WindowSize::default()
    };
    window_state.title = String::from("Papyrus Pad");

    let window_create_options = WindowCreateOptions {
        state: window_state,
        ..WindowCreateOptions::default()
    };

    let window = create_window(&mut app, window_create_options).unwrap();

    app.run(window).unwrap();
}

#[cfg(debug_assertions)]
fn create_window<T>(
    app: &mut App<T>,
    options: WindowCreateOptions<T>,
) -> Result<Window<T>, WindowCreateError> {
    use azul_theming::*;

    let interval = std::time::Duration::from_secs(1);

    hot_reload(
        "hot-reload.css",
        "hot-reload-injected.css",
        themes::dark_theme(),
        interval,
    );

    let css = css::hot_reload("hot-reload-injected.css", interval);

    app.create_hot_reload_window(options, css)
}

#[cfg(not(debug_assertions))]
fn create_window<T>(
    app: &mut App<T>,
    options: WindowCreateOptions<T>,
) -> Result<Window<T>, WindowCreateError> {
    use azul_theming::*;
    use styles::*;

    let s: String = [PAD_CSS, REPL_TERM_CSS, PATH_TREE_CSS]
        .into_iter()
        .map(|x| x.to_owned())
        .collect();

    let css = css::from_str(&inject_theme(&s, &themes::dark_theme())).unwrap();

    app.create_window(options, css)
}

#[cfg(not(debug_assertions))]
const PAD_CSS: &str = r##"
#pad {
	flex-direction: row;
}
"##;
