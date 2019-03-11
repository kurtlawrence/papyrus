extern crate azul;
extern crate cansi;
extern crate linefeed;
extern crate papyrus;

use azul::prelude::*;
use azul::widgets::label::Label;
use azul::widgets::text_input::{TextInput, TextInputState};
use linefeed::memory::MemoryTerminal;
use papyrus::*;

pub const TEST_OUTPUT: u8 = 123;

struct MyApp {
    terminal: MemoryTerminal,
    last_terminal_string: String,
    text_input: TextInputState,
}

impl Layout for MyApp {
    fn layout(&self, _: LayoutInfo<Self>) -> Dom<Self> {
        let term_str = create_terminal_string(&self.terminal);
        let categorised = cansi::categorise_text(&term_str);
        let text = cansi::construct_text_no_codes(&categorised);

        Label::new(text)
            .dom()
            .with_class("terminal")
            .with_callback(On::TextInput, Callback(on_text_input))
            .with_callback(On::VirtualKeyDown, Callback(on_vk_keydown))
            .with_tab_index(TabIndex::Auto) // make focusable
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

fn on_text_input(state: &mut AppState<MyApp>, event: &mut CallbackInfo<MyApp>) -> UpdateScreen {
    let keyboard_state = state.windows[event.window_id].get_keyboard_state();
    if let Some(ch) = keyboard_state.current_char {
        state
            .data
            .modify(|s| s.terminal.push_input(&ch.to_string()));
        Redraw
    } else {
        DontRedraw
    }
}

fn on_vk_keydown(state: &mut AppState<MyApp>, event: &mut CallbackInfo<MyApp>) -> UpdateScreen {
    let keyboard_state = state.windows[event.window_id].get_keyboard_state();
    match keyboard_state.latest_virtual_keycode {
        Some(VirtualKeyCode::Back) => {
            state.data.modify(|s| s.terminal.push_input("\x08")); // backspace character
            Redraw
        }
        Some(VirtualKeyCode::Tab) => {
            state.data.modify(|s| s.terminal.push_input("\t"));
            Redraw
        }
        Some(VirtualKeyCode::Return) => {
            state.data.modify(|s| s.terminal.push_input("\n")); // this allows the read_line() to exit
            Redraw
        }
        _ => DontRedraw,
    }
}

fn check_terminal_change(app: &mut MyApp, _: &mut AppResources) -> (UpdateScreen, TerminateDaemon) {
    let new_str = create_terminal_string(&app.terminal);
    if new_str != app.last_terminal_string {
        app.last_terminal_string = new_str;
        (Redraw, TerminateDaemon::Continue)
    } else {
        (DontRedraw, TerminateDaemon::Continue)
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
            repl = match repl.read().eval(()) {
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
                text_input: TextInputState::new(String::new()),
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
    let daemon =
        Daemon::new(check_terminal_change).with_interval(std::time::Duration::from_millis(2));
    app.add_daemon(DaemonId::new(), daemon);

    app.run(window).unwrap();
}

// put down here as it will be largeish
fn colour_slice<T: Layout>(cat_slice: &cansi::CategorisedSlice) -> Dom<T> {
    use cansi::Color as cc;
    let s = String::from_utf8_lossy(cat_slice.text_as_bytes);

    let label = Label::new(s).dom().with_class("terminal-text");
    let label = match cat_slice.fg_colour {
        cc::Cyan => {
            label.with_css_override("fg_colour", CssProperty::TextColor(StyleTextColor(CYAN)))
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
