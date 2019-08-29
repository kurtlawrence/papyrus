#[macro_use]
extern crate papyrus;

use papyrus::repl::ReadResult;

fn main() {
    let mut repl = repl!(String);

    let mut s = String::from("Hello,");

    repl.line_input(".mut"); // begin mutating block
    repl = match repl.read() {
        ReadResult::Eval(eval) => eval.eval(&mut s).repl.print(),
        ReadResult::Read(read) => read,
    };
    assert_eq!(&s, "Hello,");

    // push a string on to the mutable string, we can do this as it is mutating
    repl.line_input("app_data.push_str(\" world!\")");
    repl = match repl.read() {
        ReadResult::Eval(eval) => eval.eval(&mut s).repl.print(),
        ReadResult::Read(read) => read,
    };
    assert_eq!(&s, "Hello, world!");

    // print the string
    repl.line_input("println!(\"{}\", app_data)");
    repl = match repl.read() {
        ReadResult::Eval(eval) => eval.eval(&mut s).repl.print(),
        ReadResult::Read(read) => read,
    };

    // push more. need to initiate a mut block again
    repl.line_input(".mut"); // begin mutating block
    repl = match repl.read() {
        ReadResult::Eval(eval) => eval.eval(&mut s).repl.print(),
        ReadResult::Read(read) => read,
    };
    repl.line_input("app_data.push_str(\" I am papyrus!\")");
    match repl.read() {
        ReadResult::Eval(eval) => eval.eval(&mut s).repl.print(),
        ReadResult::Read(read) => read,
    };
    assert_eq!(&s, "Hello, world! I am papyrus!");
}
