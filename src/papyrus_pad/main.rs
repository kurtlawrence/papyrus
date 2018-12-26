extern crate azul;
extern crate linefeed;

use azul::prelude::*;
use azul::widgets::{
    button::Button, label::Label, text_input::TextInput, text_input::TextInputState,
};
use linefeed::memory::MemoryTerminal;

struct MyApp {
    outputs: String,
    input: TextInputState,
    terminal: MemoryTerminal,
}

impl Layout for MyApp {
    fn layout(&self, info: WindowInfo<Self>) -> Dom<Self> {
        let term_str = create_terminal_string(&self.terminal);
        let terminal = Label::new(term_str).dom().with_class("terminal");
        let label = Label::new(self.input.text.clone()).dom();
        let input = TextInput::new()
            .bind(info.window, &self.input, self)
            .dom(&self.input)
            .with_callback(On::VirtualKeyDown, Callback(input_to_terminal));
        Dom::new(NodeType::Div)
            .with_child(terminal)
            .with_child(label)
            .with_child(input)
    }
}

fn create_terminal_string(term: &MemoryTerminal) -> String {
	 let mut term_str = String::new();
	let mut lines = term.lines();
	while let Some(chars) = lines.next() {
		for c in chars {
			term_str.push(*c);
		}
		term_str.push('\n');
	}
	term_str
}

fn input_to_terminal(state: &mut AppState<MyApp>, event: WindowEvent<MyApp>) -> UpdateScreen {
    let keyboard_state = state.windows[event.window].get_keyboard_state();
    if let Some(keycode) = keyboard_state.latest_virtual_keycode {
        if keycode == VirtualKeyCode::Return {
            state.data.modify(|s| {
                let input_str = &s.input.text;
				println!("{}", input_str);
				println!("hit", );
                s.terminal.write(input_str);
				println!("{}", create_terminal_string(&s.terminal));
                s.input = TextInputState::new(String::new());
            });
            return UpdateScreen::Redraw;
        }
    }
    UpdateScreen::DontRedraw
}

fn main() {
    println!("hello world",);

    let app = {
        App::new(
            MyApp {
                outputs: String::new(),
                input: TextInputState::new(String::new()),
                terminal: MemoryTerminal::new(),
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
            css::hot_reload("styles/test.css", true),
        )
        .unwrap()
		// Window::new(WindowCreateOptions::default(), css::native()).unwrap()
    } else {
        Window::new(WindowCreateOptions::default(), css::native()).unwrap()
    };
    app.run(window).unwrap();
}

// fn update_counter(state: &mut AppState<MyApp>, event: WindowEvent<MyApp>) -> UpdateScreen {
// 	state.data.modify(|s| s.counter += 1);
// 	UpdateScreen::Redraw
// }
