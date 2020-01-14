use super::{run, Screen};
use crate::run::RunCallbacks;
use crossbeam_channel::{unbounded, Receiver, Sender};
use crossterm as xterm;
use std::thread::JoinHandle;
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

struct Tx(Sender<Event>);

impl Tx {
    fn text(&self, text: &str) -> &Self {
        for ch in text.chars() {
            self.0
                .send(Event::Key(KeyEvent::new(
                    KeyCode::Char(ch),
                    KeyModifiers::empty(),
                )))
                .unwrap();
        }
        self
    }

    fn enter(&self) -> &Self {
        self.0
            .send(Event::Key(KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::empty(),
            )))
            .unwrap();
        self
    }

    fn tab(&self) -> &Self {
        self.0
            .send(Event::Key(KeyEvent::new(
                KeyCode::Tab,
                KeyModifiers::empty(),
            )))
            .unwrap();
        self
    }

    fn ctrl(&self, ch: char) -> &Self {
        self.0
            .send(Event::Key(KeyEvent::new(
                KeyCode::Char(ch),
                KeyModifiers::CONTROL,
            )))
            .unwrap();
        self
    }
}

impl Drop for Tx {
    fn drop(&mut self) {
        self.text(":exit").enter();
        slp(50);
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
