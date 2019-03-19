#[macro_use]
extern crate papyrus;

use azul::prelude::*;
use linefeed::memory::MemoryTerminal;
use papyrus::prelude::*;
use papyrus::widgets::{self, pad::*};

struct MyApp {
    terminal: MemoryTerminal,
    last_terminal_string: String,
    pad: PadState,
}

impl Layout for MyApp {
    fn layout(&self, info: LayoutInfo<Self>) -> Dom<Self> {
        Dom::div().with_child(Pad::new(info.window, &self.pad, &self).dom(&self.pad))
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

// fn on_text_input(state: &mut AppState<MyApp>, event: &mut CallbackInfo<MyApp>) -> UpdateScreen {
//     let ch = state.windows[event.window_id].get_keyboard_state().current_char?;
//     state.data.modify(|s| s.handle_input(ch));
//     Redraw
// }

// fn on_vk_keydown(state: &mut AppState<MyApp>, event: &mut CallbackInfo<MyApp>) -> UpdateScreen {
//     let keyboard_state = state.windows[event.window_id].get_keyboard_state();
//     match keyboard_state.latest_virtual_keycode {
//         Some(VirtualKeyCode::Back) => {
//             state.data.modify(|s| s.handle_input('\x08')); // backspace character
//             Redraw
//         }
//         Some(VirtualKeyCode::Tab) => {
//             state.data.modify(|s| s.handle_input('\t'));
//             Redraw
//         }
//         Some(VirtualKeyCode::Return) => {
//             state.data.modify(|s| s.handle_input('\n')); // this allows the read_line() to exit
//             Redraw
//         }
//         _ => DontRedraw,
//     }
// }

fn check_terminal_change(app: &mut MyApp, _: &mut AppResources) -> (UpdateScreen, TerminateDaemon) {
    let new_str = create_terminal_string(&app.terminal);
    if new_str != app.last_terminal_string {
        app.last_terminal_string = new_str;
        (Redraw, TerminateDaemon::Continue)
    } else {
        (Redraw, TerminateDaemon::Continue)
    }
}

// fn check_evaluating_done(app: &mut MyApp, _: &mut AppResources) -> (UpdateScreen, TerminateDaemon) {
//     let done = app.eval.as_ref().map_or(false, |e| e.completed());
//     if done {
//         let eval = app.eval.take().expect("should be some");
//         app.repl = Some(
//             eval.wait()
//                 .expect("got an eval signal, which I have not handled yet")
//                 .print(),
//         );
//         (Redraw, TerminateDaemon::Continue)
//     } else {
//         (DontRedraw, TerminateDaemon::Continue)
//     }
// }

// impl MyApp {
//     fn handle_input(&mut self, input: char) {
//         if self.eval.is_none() {
//             let repl = self
//                 .repl
//                 .take()
//                 .expect("repl was empty, which would indicate a broken state?");
//             match repl.push_input(input) {
//                 repl::PushResult::Read(r) => self.repl = Some(r),
//                 repl::PushResult::Eval(r) => self.eval = Some(r.eval_async(())),
//             }
//         }
//     }
// }

fn main() {
    let term = MemoryTerminal::new();
    let closure_term = term.clone();

    // std::thread::spawn(move || {
    // 	let mut repl = repl_with_term!(closure_term);
    //     loop {
    //         repl = match repl.read().eval(()) {
    //             Ok(print) => print.print(),
    //             Err(_) => break, // this will stop the repl if we get here
    //         };
    //     }
    // });

    let repl = repl_with_term!(term.clone());

    let mut app = {
        App::new(
            MyApp {
                terminal: term,
                last_terminal_string: String::new(),
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

    // let daemon =
    //     Daemon::new(check_evaluating_done).with_interval(std::time::Duration::from_millis(2));
    // app.add_daemon(DaemonId::new(), daemon);

    app.run(window).unwrap();
}

// put down here as it will be largeish
fn colour_slice<T: Layout>(cat_slice: &cansi::CategorisedSlice) -> Dom<T> {
    const PROPERTY_STR: &str = "ansi_esc_color";
    let s = String::from_utf8_lossy(cat_slice.text_as_bytes);

    Dom::label(s).with_class("terminal-text").with_css_override(
        PROPERTY_STR,
        StyleTextColor(widgets::colour::map(&cat_slice.fg_colour)).into(),
    )
}
