extern crate azul;
extern crate cansi;
extern crate linefeed;
extern crate papyrus;

use azul::prelude::*;
use azul::widgets::label::Label;
use cansi::*;
use linefeed::memory::MemoryTerminal;

use papyrus::*;

pub const TEST_OUTPUT: u8 = 123;

struct MyApp {
    terminal: MemoryTerminal,
    last_terminal_string: String,
}

impl Layout for MyApp {
    fn layout(&self, _: WindowInfo<Self>) -> Dom<Self> {
        let term_str = create_terminal_string(&self.terminal);
        // println!("{}", String::from_utf8_lossy(term_str.as_bytes()));
        let categorised = cansi::categorise_text(&term_str);
        let text = construct_text_no_codes(&categorised);
        // println!("{}", text);
        let dom = Dom::new(NodeType::Div)
            .with_class("terminal")
            .with_callback(On::TextInput, Callback(on_text_input))
            .with_callback(On::VirtualKeyDown, Callback(on_vk_keydown));

        dom.with_child(Label::new(text).dom().with_class("terminal"))
    }
}

fn create_terminal_string(term: &MemoryTerminal) -> String {
    let mut string = String::new();
    let mut lines = term.lines();
    while let Some(chars) = lines.next() {
        for ch in chars {
            string.push(*ch);
        }
        string.push('\n');
    }
    string
}

fn on_text_input(state: &mut AppState<MyApp>, event: WindowEvent<MyApp>) -> UpdateScreen {
    let keyboard_state = state.windows[event.window].get_keyboard_state();
    if let Some(ch) = keyboard_state.current_char {
        state
            .data
            .modify(|s| s.terminal.push_input(&ch.to_string()));
        UpdateScreen::Redraw
    } else {
        UpdateScreen::DontRedraw
    }
}

fn on_vk_keydown(state: &mut AppState<MyApp>, event: WindowEvent<MyApp>) -> UpdateScreen {
    let keyboard_state = state.windows[event.window].get_keyboard_state();
    match keyboard_state.latest_virtual_keycode {
        Some(VirtualKeyCode::Back) => {
            state.data.modify(|s| s.terminal.push_input("\x08")); // backspace character
            UpdateScreen::Redraw
        }
        Some(VirtualKeyCode::Tab) => {
            state.data.modify(|s| s.terminal.push_input("\t"));
            UpdateScreen::Redraw
        }
        Some(VirtualKeyCode::Return) => {
            state.data.modify(|s| s.terminal.push_input("\n")); // this allows the read_line() to exit
            UpdateScreen::Redraw
        }
        _ => UpdateScreen::DontRedraw,
    }
}

fn check_terminal_change(app: &mut MyApp, _: &mut AppResources) -> (UpdateScreen, TerminateDaemon) {
    let new_str = create_terminal_string(&app.terminal);
    if new_str != app.last_terminal_string {
        app.last_terminal_string = new_str;
        (UpdateScreen::Redraw, TerminateDaemon::Continue)
    } else {
        (UpdateScreen::DontRedraw, TerminateDaemon::Continue)
    }
}

fn main() {
    println!("hello world",);

    let term = MemoryTerminal::new();
    let closure_term = term.clone();

    std::thread::spawn(move || {
        let mut repl_data = ReplData::default();
        let terminal = closure_term.clone();
        let mut repl = Repl::with_term(terminal.clone(), &mut repl_data);
        loop {
            repl = match repl.read().eval() {
                Ok(print) => print.print(),
                Err(_) => break, // this will stop the repl if we get here
            };
        }
    });

    let mut app = {
        App::new(
            MyApp {
                terminal: term,
                last_terminal_string: String::new(),
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
                std::time::Duration::from_millis(500),
            ),
        )
        .unwrap()
    // Window::new(WindowCreateOptions::default(), css::native()).unwrap()
    } else {
        Window::new(WindowCreateOptions::default(), css::native()).unwrap()
    };
    let daemon = Daemon::unique(DaemonCallback(check_terminal_change))
        .run_every(std::time::Duration::from_millis(2));
    app.add_daemon(daemon);

    app.run(window).unwrap();
}

// put down here as it will be largeish
fn colour_slice<T: Layout>(cat_slice: &CategorisedSlice) -> Dom<T> {
    let s = String::from_utf8_lossy(cat_slice.text_as_bytes);

    let label = Label::new(s).dom().with_class("terminal-text");
    let label = match cat_slice.fg_colour {
        Color::Cyan => {
            label.with_style_override("fg_colour", CssProperty::TextColor(StyleTextColor(CYAN)))
        }
        _ => label,
    };

    label
}

const CYAN: ColorU = ColorU {
    r: 0,
    b: 170,
    g: 170,
    a: 0,
};
