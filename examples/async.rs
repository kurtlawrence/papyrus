#[macro_use]
extern crate papyrus;

use std::sync::{Arc, Mutex};

fn main() {
	let mut repl = repl!(String);

	let mut v = Arc::new(Mutex::new(String::new()));

	let line = "std::thread::sleep_ms(5000); app_data.push_str(\"Hello, world!\"); app_data\n";

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

	let v_lock = v.lock().unwrap();
	assert_eq!(&v_lock.to_string(), "Hello, world!");
	// TODO this is currently broken -- app_data is no longer mutable, but if mechanism in place to mutate might alter.
}
