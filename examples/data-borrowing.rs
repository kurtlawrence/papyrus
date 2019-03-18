#[macro_use]
extern crate papyrus;
extern crate linefeed;

use papyrus::*;

fn main() {
	// first, lets show how to pass through a simple number
	let v = 123;

	let data = repl_data!(u32);
	let mut repl = papyrus::Repl::default_terminal(data);

	repl = execute_line(repl, v, "app_data");
	execute_line(repl, v, "app_data + 123");

	// second, how about a borrowed value?
	let v = String::from("Hello, world!");

	let data = repl_data!(&String);
	let mut repl = papyrus::Repl::default_terminal(data);

	repl = execute_line(repl, &v, "app_data"); // <-- borrowed!
	execute_line(repl, &v, "app_data.to_uppercase()");

	// third, lets try a mutable borrow
	let mut v = String::from("Hello,");

	let data = repl_data!(&mut String);
	let mut repl = papyrus::Repl::default_terminal(data);

	// for ch in ", world!".chars() {
	// 	v.push(ch);
	// }

	// println!("{}", v);

	for ch in "app_data\n".chars() {
		let tmp = &mut v;
		repl = match repl.push_input(ch) {
			papyrus::repl::PushResult::Read(r) => r,
			papyrus::repl::PushResult::Eval(r) => r.eval(tmp).unwrap().print(),
		}
	}
}

/// Adds the newline for us!
fn execute_line<T: linefeed::Terminal + 'static, D: Copy>(
	repl: Repl<repl::Read, T, D>,
	app_data: D,
	line: &str,
) -> Repl<repl::Read, T, D> {
	let mut repl = repl;
	for ch in line.chars().into_iter().chain("\n".chars()) {
		repl = match repl.push_input(ch) {
			papyrus::repl::PushResult::Read(r) => r,
			papyrus::repl::PushResult::Eval(r) => r.eval(app_data).unwrap().print(),
		}
	}

	repl
}
