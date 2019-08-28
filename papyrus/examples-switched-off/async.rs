#[macro_use]
extern crate papyrus;

use papyrus::repl::ReadResult;
use std::sync::{Arc, RwLock};

fn main() {
    let mut repl = repl!(String);

    let v = Arc::new(RwLock::new(String::new()));

    repl.line_input(".mut");
    repl = match repl.read() {
        ReadResult::Eval(eval) => eval.eval_async(&v).wait().repl.print(),
        ReadResult::Read(read) => read,
    };

    let line = "std::thread::sleep_ms(5000); app_data.push_str(\"Hello, world!\"); app_data";

    repl.line_input(line);

    let eval = match repl.read() {
        ReadResult::Eval(eval) => eval.eval_async(&v),
        _ => unreachable!(),
    };

    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("evaluating on another thread"); // <- this might muck up the output, as it is now multi-threaded!

    if !eval.completed() {
        std::thread::sleep(std::time::Duration::from_secs(2));
        println!("still evaluating...");
    }

    eval.wait().repl.print();

    let v_lock = v.read().unwrap();
    assert_eq!(&v_lock.to_string(), "Hello, world!");
}
