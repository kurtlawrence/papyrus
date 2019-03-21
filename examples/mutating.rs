#[macro_use]
extern crate papyrus;

fn main() {
	let repl = repl!(String);

	let mut s = String::from("Hello,");

	let repl = repl.push_input_str(".mut\n").unwrap().0; // begin mutating block
	let repl = repl.eval(&mut s).repl.print();
	assert_eq!(&s, "Hello,");

	// push a string on to the mutable string, we can do this as it is mutating
	let repl = repl
		.push_input_str("app_data.push_str(\" world!\")\n")
		.unwrap()
		.0; // try to push onto it
	let repl = repl.eval(&mut s).repl.print();
	assert_eq!(&s, "Hello, world!");

	// print the string
	let repl = repl
		.push_input_str("println!(\"{}\", app_data)\n")
		.unwrap()
		.0;
	let repl = repl.eval(&mut s).repl.print();

	// push more. need to initiate a mut block again
	repl.push_input_str(".mut\n")
		.unwrap()
		.0
		.eval(&mut s)
		.repl
		.print()
		.push_input_str("app_data.push_str(\" I am papyrus!\")\n")
		.unwrap()
		.0
		.eval(&mut s)
		.repl
		.print();
	assert_eq!(&s, "Hello, world! I am papyrus!");
}
