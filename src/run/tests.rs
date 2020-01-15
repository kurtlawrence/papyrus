use super::{run, InputBuffer, Screen};
use crate::run::RunCallbacks;
use crossbeam_channel::{unbounded, Receiver, Sender};
use crossterm as xterm;
use std::{
    io::{self, Write},
    thread::JoinHandle,
};
use xterm::{event::*, Result};

macro_rules! assert_eq {
    ($lhs:expr, $rhs:expr) => {{
        assert_eq!($lhs, $rhs, "")
    }};
    ($lhs:expr, $rhs:expr, $($arg:tt)*) => {{
        if $lhs != $rhs {
            eprintln!("lhs does not equal rhs;");
            eprintln!("lhs: {:?}", $lhs);
            eprintln!("rhs: {:?}", $rhs);
            eprintln!($($arg)*);
            std::assert_eq!($lhs, $rhs, $($arg)*);
        }
    }}
}

// REPL INTEGRATION TESTS -----------------------------------------------------
#[test]
fn entry_and_exit() {
    colour_off();
    let (tx, rx) = unbounded();
    let tx = Tx(tx);
    let jh = fire_off_run(rx);

    let result = finish_repl(jh, tx);

    println!("{}", result);

    let expected = r#"[lib] papyrus=> :exit
[lib] papyrus=> "#;

    assert_eq!(result, expected);

    colored::control::unset_override();
}

#[test]
fn cusor_positions() {
    let (tx, rx) = unbounded();
    let tx = Tx(tx);
    let jh = fire_off_run(rx);

    slp(100);
    assert_eq!(
        col(),
        16,
        "expecting cursor to be just after '[lib] papyrus=> '"
    );

    tx.text("let apple = 1;").enter().text("app").tab();
    slp(100);
    assert_eq!(
        col(),
        21,
        "expecting cursor to be after '[lib] papyrus.> apple'"
    );

    tx.enter();
}

#[test]
fn verbatim_mode_tab_input() {
    colour_off();
    let (tx, rx) = unbounded();
    let tx = Tx(tx);
    let jh = fire_off_run(rx);

    tx.ctrl('o').tab();
    slp(100);

    assert_eq!(col(), 24, "tab size 8");
    tx.ctrl('d').enter();

    let result = finish_repl(jh, tx);
    println!("{}", result);
    let expected = "[lib] papyrus=> \t
[lib] papyrus=> 
[lib] papyrus=> :exit
[lib] papyrus=> ";
    assert_eq!(result, expected);
}

#[test]
fn backspace_past_start() {
    colour_off();
    let (tx, rx) = unbounded();
    let tx = Tx(tx);
    let jh = fire_off_run(rx);

    tx.text("12345").backspace(10).enter();

    let result = finish_repl(jh, tx);
    println!("{}", result);
    let expected = "[lib] papyrus=> 
[lib] papyrus=> :exit
[lib] papyrus=> ";
    assert_eq!(result, expected);
}

#[test]
fn test_cursor_moving() {
    colour_off();
    let (tx, rx) = unbounded();
    let tx = Tx(tx);
    let jh = fire_off_run(rx);

    tx.text("let appe = 1;");
    slp(100);
    assert_eq!(col(), 29, "cursor position should be after 'let appe = 1;'");

    tx.left(6);
    slp(50);
    assert_eq!(col(), 23);

    tx.text("l");
    slp(100);
    assert_eq!(col(), 24, "cursor should only progress with inserted text");

    tx.left(100);
    slp(100);
    assert_eq!(col(), 16, "col be at prompt start");

    tx.right(100);
    slp(100);
    assert_eq!(col(), 30);

    tx.enter();
    slp(100);
    assert_eq!(col(), 16, "cursor should be after prompt");

    let result = finish_repl(jh, tx);
    println!("{}", result);
    let expected = "[lib] papyrus=> let apple = 1;
[lib] papyrus.> :exit
[lib] papyrus=> ";
    assert_eq!(result, expected);
}

#[test]
fn test_input_inside_line() {
    colour_off();
    let (tx, rx) = unbounded();
    let tx = Tx(tx);
    let jh = fire_off_run(rx);

    tx.text("let apple = 1;");
    slp(100);
    assert_eq!(
        col(),
        30,
        "cursor position should be after 'let apple = 1;'"
    );

    tx.left(5);
    slp(50);
    assert_eq!(col(), 25);

    tx.backspace(5);
    slp(100);
    assert_eq!(col(), 20);

    tx.text("banana");
    slp(100);
    assert_eq!(col(), 26);

    tx.enter();
    slp(100);
    assert_eq!(col(), 16, "cursor should be after prompt");

    let result = finish_repl(jh, tx);
    println!("{}", result);
    let expected = "[lib] papyrus=> let banana = 1;
[lib] papyrus.> :exit
[lib] papyrus=> ";
    assert_eq!(result, expected);
}

// INTERFACE INTEGRATION TESTS ------------------------------------------------
#[test]
fn interface_integration() {
    let (tx, rx) = unbounded();
    let tx = Tx(tx);
    let mut inputbuf = InputBuffer::new();
    let mut screen = Screen(rx);
    writeln!(io::stdout()).unwrap();
    slp(150);
    let mut interface = screen.begin_interface_input(&mut inputbuf).unwrap();

    // Use <C-+> to send the end signal
    let end: &[Event] = &[Event::Key(KeyEvent::new(
        KeyCode::Char('+'),
        KeyModifiers::CONTROL,
    ))];

    tx.text("Hello").ctrl('+');
    interface.read_until(end);
    slp(100);
    assert_eq!(col(), 5);
    assert_eq!(interface.buf_pos(), 5);

    tx.text("\n").ctrl('+');
    interface.read_until(end);
    slp(100);
    assert_eq!(col(), 0);
    assert_eq!(interface.buf_pos(), 6);

    tx.text("Hello\nWorld!").ctrl('+');
    interface.read_until(end);
    slp(100);
    assert_eq!(col(), 6, "end of 'World!'");
    assert_eq!(interface.buf_pos(), 18);
}

struct Tx(Sender<Event>);

impl Tx {
    fn send(&self, ev: Event) {
        self.0.send(ev).ok();
    }

    fn text(&self, text: &str) -> &Self {
        for ch in text.chars() {
            self.send(Event::Key(KeyEvent::new(
                KeyCode::Char(ch),
                KeyModifiers::empty(),
            )));
        }
        self
    }

    fn enter(&self) -> &Self {
        self.send(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::empty(),
        )));
        self
    }

    fn tab(&self) -> &Self {
        self.send(Event::Key(KeyEvent::new(
            KeyCode::Tab,
            KeyModifiers::empty(),
        )));
        self
    }

    fn ctrl(&self, ch: char) -> &Self {
        self.send(Event::Key(KeyEvent::new(
            KeyCode::Char(ch),
            KeyModifiers::CONTROL,
        )));
        self
    }

    fn backspace(&self, n: usize) -> &Self {
        for _ in 0..n {
            self.send(Event::Key(KeyEvent::new(
                KeyCode::Backspace,
                KeyModifiers::empty(),
            )));
        }
        self
    }

    fn left(&self, n: usize) -> &Self {
        for _ in 0..n {
            self.send(Event::Key(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::empty(),
            )));
        }
        self
    }

    fn right(&self, n: usize) -> &Self {
        for _ in 0..n {
            self.send(Event::Key(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::empty(),
            )));
        }
        self
    }
}

impl Drop for Tx {
    fn drop(&mut self) {
        self.text(":exit").enter();
        slp(200);
    }
}

fn col() -> u16 {
    xterm::cursor::position().unwrap().0
}

fn slp(millis: u64) {
    std::thread::sleep(std::time::Duration::from_millis(millis));
}

fn colour_off() {
    colored::control::set_override(false);
}

fn fire_off_run(rx: Receiver<Event>) -> JoinHandle<Result<String>> {
    std::thread::spawn(|| {
        let screen = Screen(rx);
        let repl = crate::repl::Repl::<_, ()>::default();
        run(repl, RunCallbacks::new(&mut ()), || Ok(screen))
    })
}

fn finish_repl(jh: JoinHandle<Result<String>>, tx: Tx) -> String {
    drop(tx);
    jh.join().unwrap().unwrap()
}
