#[macro_use]
extern crate papyrus;

use std::sync::{Arc, RwLock};

fn main() {
    let mut repl = repl!(String);

    let mut v = Arc::new(RwLock::new(String::new()));

    // notice the .mut\n putting into mutating state
    let line =
        ".mut\nstd::thread::sleep_ms(5000); app_data.push_str(\"Hello, world!\"); app_data\n";

    let mut slice = line;
    while slice.len() > 0 {
        let (eval, s) = repl.push_input_str(slice).unwrap();
        slice = s;
        let eval = eval.eval_async(&mut v);
        println!("evaluating on another thread"); // <- this might muck up the output, as it is now multi-threaded!
        if !eval.completed() {
            std::thread::sleep(std::time::Duration::from_secs(2));
            println!("still evaluating...");
        }
        repl = eval.wait().repl.print();
    }

    let v_lock = v.read().unwrap();
    assert_eq!(&v_lock.to_string(), "Hello, world!");
}