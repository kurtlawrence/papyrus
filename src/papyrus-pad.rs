extern crate azul;
extern crate linefeed;
extern crate papyrus;

use azul::prelude::*;
use azul::widgets::{
    button::Button, label::Label, text_input::TextInput, text_input::TextInputState,
};
use linefeed::interface::Interface;
use linefeed::memory::MemoryTerminal;
use linefeed::reader::ReadResult;
use linefeed::terminal::{Terminal, TerminalReader, TerminalWriter};
use papyrus::*;

struct MyApp {
    input: TextInputState,
    terminal: MemoryTerminal,
    reader: InputReader<MemoryTerminal>,
    repl_data: ReplData,
}

impl Layout for MyApp {
    fn layout(&self, info: WindowInfo<Self>) -> Dom<Self> {
        let term_str = create_terminal_string(&self.terminal);
        let terminal = Label::new(term_str).dom().with_class("terminal");
        let input = TextInput::new()
            .bind(info.window, &self.input, self)
            .dom(&self.input)
            .with_callback(On::TextInput, Callback(on_text_input))
            .with_callback(On::VirtualKeyDown, Callback(on_vk_keydown));
        Dom::new(NodeType::Div)
            .with_child(terminal)
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

fn on_text_input(state: &mut AppState<MyApp>, event: WindowEvent<MyApp>) -> UpdateScreen {
    let keyboard_state = state.windows[event.window].get_keyboard_state();
    if let Some(ch) = keyboard_state.current_char {
        state
            .data
            .modify(|state| state.terminal.push_input(&ch.to_string()));
        UpdateScreen::Redraw
    } else {
        UpdateScreen::DontRedraw
    }
}

fn on_vk_keydown(state: &mut AppState<MyApp>, event: WindowEvent<MyApp>) -> UpdateScreen {
    let keyboard_state = state.windows[event.window].get_keyboard_state();
    match keyboard_state.latest_virtual_keycode {
        Some(VirtualKeyCode::Back) => {
            state.data.modify(|state| {
                let mut buf = [0];
                state.terminal.read_input(&mut buf);
            });
            UpdateScreen::Redraw
        }
        Some(VirtualKeyCode::Return) => {
            state.data.modify(|s| {
                let input_str = &s.input.text;
                println!("{}", input_str);
                println!("hit",);
                s.terminal.push_input("\n"); // this allows the read_line() to exit

                match Repl::new(&mut s.repl_data).read(&mut s.reader).eval() {
                    Ok(print) => {
                        print.print();
                    }
                    Err(_) => (), // do nothing if asked to exit...
                }

                println!("{}", create_terminal_string(&s.terminal));
                s.input = TextInputState::new(String::new());
            });
            UpdateScreen::Redraw
        }
        _ => UpdateScreen::DontRedraw,
    }
}

fn main() {
    println!("hello world",);

    let term = MemoryTerminal::new();
    let term_clone = term.clone();
    let reader =
        InputReader::with_term("papyrus-pad-terminal", term).expect("failed to initialise reader");
    let repl_data = ReplData::default();

    let app = {
        App::new(
            MyApp {
                input: TextInputState::new(String::new()),
                terminal: term_clone,
                reader: reader,
                repl_data: repl_data,
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

// mod my_terminal {

// struct MyTerminal<'a> {
// 	name: &'a str
// }

// impl<'a> Terminal for MyTerminal<'a> {

// 	type PrepareState = ();

// 	fn name(&self) -> &str {
// 		self.name
// 	}

// 	fn lock_read(&self) -> Box<dyn TerminalReader<Self>> {

// 	}

// 	fn lock_write(&self) -> Box<dyn TerminalWriter<Self>> {

// 	}

// }

// }
