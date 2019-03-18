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
}
