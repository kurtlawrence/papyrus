#[macro_use]
extern crate papyrus;

use azul::prelude::*;
use linefeed::memory::MemoryTerminal;
use papyrus::widgets::pad::*;

struct MyApp {
    pad: PadState,
}

impl GetPad for MyApp {
    fn pad_state(&mut self) -> &mut PadState {
        &mut self.pad
    }
}

impl Layout for MyApp {
    fn layout(&self, info: LayoutInfo<Self>) -> Dom<Self> {
        Dom::div().with_child(Pad::new(info.window, &self.pad, &self).dom(&self.pad))
    }
}

fn main() {
    let term = MemoryTerminal::new();

    let repl = repl_with_term!(term.clone());

    let app = {
        App::new(
            MyApp {
                pad: PadState::new(repl),
            },
            AppConfig {
                enable_logging: Some(LevelFilter::Error),
                log_file_path: Some("debug.log".to_string()),
                ..Default::default()
            },
        )
    };
    let window = if cfg!(debug_assertions) {
        Window::new_hot_reload(
            WindowCreateOptions::default(),
            css::hot_reload_override_native(
                "styles/test.css",
                std::time::Duration::from_millis(1000),
            ),
        )
        .unwrap()
    // Window::new(WindowCreateOptions::default(), css::native()).unwrap()
    } else {
        Window::new(WindowCreateOptions::default(), css::native()).unwrap()
    };

    app.run(window).unwrap();
}
