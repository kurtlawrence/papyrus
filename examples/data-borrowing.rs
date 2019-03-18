#[macro_use]
extern crate papyrus;
extern crate linefeed;

use papyrus::*;

fn main() {
	// first, lets show how to pass through a simple number
	let v = 123;

	let mut repl = repl!(u32);

	repl = execute_line(repl, v, "app_data");
	execute_line(repl, v, "app_data + 123");

	// second, how about a borrowed value?
	let v = String::from("Hello, world!");

	let mut repl = repl!(&String);

	repl = execute_line(repl, &v, "app_data"); // <-- borrowed!
	execute_line(repl, &v, "app_data.to_uppercase()");

	// third, lets try a mutable borrow
	// TODO FIX THIS!
	// let mut v = String::from("Hello,");

	// let data = repl_data!(&mut String);
	// let mut repl = papyrus::Repl::default_terminal(data);

	// repl = { execute_line(repl, &mut v, "app_data") }; // <-- borrow mutably!
	// {
	// 	// assert_eq!(&v, "Hello,");
	// }
	// execute_line(repl, &mut v, r#"app_data.push_str(" world!"); app_data"#);
	// // assert_eq!(&v, "Hello, world!");
}

/// Adds the newline for us!
fn execute_line<T: linefeed::Terminal + 'static, D, R>(
	repl: Repl<repl::Read, T, D, R>,
	app_data: D,
	line: &str,
) -> Repl<repl::Read, T, D, R> {
	match read_until_new_line(repl, line).map(|eval| eval.eval(app_data).unwrap().print()) {
		Ok(repl) => repl,
		Err(repl) => repl,
	}
}

/// Adds the newline for us!
fn read_until_new_line<T: linefeed::Terminal + 'static, D, R>(
	repl: Repl<repl::Read, T, D, R>,
	line: &str,
) -> Result<Repl<repl::Evaluate, T, D, R>, Repl<repl::Read, T, D, R>> {
	let mut repl = repl;
	for ch in line.chars().into_iter().chain("\n".chars()) {
		repl = match repl.push_input(ch) {
			papyrus::repl::PushResult::Read(r) => r,
			papyrus::repl::PushResult::Eval(r) => return Ok(r),
		}
	}

	Err(repl)
}
