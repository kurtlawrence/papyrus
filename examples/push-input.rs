#[macro_use]
extern crate papyrus;

fn main() {
	let mut repl = repl!();

	let input = "1+2\n";

	for ch in input.chars() {
		std::thread::sleep(std::time::Duration::from_secs(1)); // sleep a little to show it inputing!
		match repl.push_input(ch) {
			papyrus::repl::PushResult::Read(r) => repl = r,
			papyrus::repl::PushResult::Eval(r) => repl = r.eval(()).unwrap().print(),
		}
	}

	// now lets try defining a function and running it!
	let code = r#"fn hello() -> &'static str {
"hello"
}
hello()
"#; // notice trailing new line

	for ch in code.chars() {
		std::thread::sleep(std::time::Duration::from_millis(100)); // sleep a little to show it inputing!
		match repl.push_input(ch) {
			papyrus::repl::PushResult::Read(r) => repl = r,
			papyrus::repl::PushResult::Eval(r) => repl = r.eval(()).unwrap().print(),
		}
	}

	// there is another way to handle this, we can send through a block of code and get it to eval straight away
	// A slice is returned of the unread code, meaning you can place into a loop
	// I am eliding checking of EvalSignals and the push_input_str
	let code = r#"fn hello2() -> &'static str {
"hello2"
}
hello2()
"#;

	let mut slice = code;
	while slice.len() > 0 {
		let (eval, s) = repl.push_input_str(slice).unwrap();
		slice = s;
		repl = eval.eval(()).unwrap().print();
	}
}
