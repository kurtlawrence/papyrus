use super::{run, Screen};
use crate::run::RunCallbacks;
use crossbeam_channel::{unbounded, Receiver, Sender};
use crossterm as xterm;
use std::thread::JoinHandle;
use xterm::{event::*, Result};

#[test]
fn entry_and_exit() {
    colored::control::set_override(false);

    let (tx, rx) = unbounded();

    let tx = Tx(tx);
    let jh = fire_off_run(rx);

    let result = finish_repl(jh, tx);

    println!("{}", result);

    let expected = r#"[lib] papyrus=> :cancel
cancelled input and returned to root
[lib] papyrus=> :cancel
cancelled input and returned to root
[lib] papyrus=> :exit
[lib] papyrus=> "#;

    assert_eq!(result, expected);

    colored::control::unset_override();
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
}

fn fire_off_run(rx: Receiver<Event>) -> JoinHandle<Result<String>> {
    std::thread::spawn(|| {
        let screen = Screen(rx);
        let repl = crate::repl::Repl::<_, ()>::default();
        run(repl, RunCallbacks::new(&mut ()), || Ok(screen))
    })
}

fn finish_repl(jh: JoinHandle<Result<String>>, tx: Tx) -> String {
    tx.text(":cancel")
        .enter()
        .text(":cancel")
        .enter()
        .text(":exit")
        .enter();
    jh.join().unwrap().unwrap()
}
