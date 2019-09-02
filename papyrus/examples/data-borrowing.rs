fn main() {}
// #[macro_use]
// extern crate papyrus;
//
// use papyrus::prelude::*;
// use repl::ReadResult;
//
// fn main() {
//     // first, lets show how to pass through a simple number
//     let mut v = 123;
//
//     let mut repl = repl!(u32);
//
//     repl.line_input("app_data");
//     repl = match repl.read() {
//         ReadResult::Eval(eval) => eval.eval(&mut v).repl.print(),
//         ReadResult::Read(read) => read,
//     };
//
//     repl.line_input("app_data + 123");
//     match repl.read() {
//         ReadResult::Eval(eval) => eval.eval(&mut v).repl.print(),
//         ReadResult::Read(read) => read,
//     };
//
//     // second, how about a borrowed value?
//     let mut v = String::from("Hello, world!");
//
//     let mut repl = repl!(String);
//
//     // v borrowed!
//     repl.line_input("app_data");
//     repl = match repl.read() {
//         ReadResult::Eval(eval) => eval.eval(&mut v).repl.print(),
//         ReadResult::Read(read) => read,
//     };
//
//     repl.line_input("app_data.to_uppercase()");
//     match repl.read() {
//         ReadResult::Eval(eval) => eval.eval(&mut v).repl.print(),
//         ReadResult::Read(read) => read,
//     };
//
//     // third, lets try a mutable borrow
//     let mut v = String::from("Hello,");
//
//     let mut repl = repl!(String);
//
//     // v mutably borrowed!
//     repl.line_input("app_data.to_string()");
//     repl = match repl.read() {
//         ReadResult::Eval(eval) => eval.eval(&mut v).repl.print(),
//         ReadResult::Read(read) => read,
//     };
//     assert_eq!(&v, "Hello,");
//
//     // get into mutating block
//     repl.line_input(".mut");
//     repl = match repl.read() {
//         ReadResult::Eval(eval) => eval.eval(&mut v).repl.print(),
//         ReadResult::Read(read) => read,
//     };
//
//     // now change the string
//     repl.line_input(r#"app_data.push_str(" world!"); app_data"#);
//     match repl.read() {
//         ReadResult::Eval(eval) => eval.eval(&mut v).repl.print(),
//         ReadResult::Read(read) => read,
//     };
//     assert_eq!(&v, "Hello, world!");
// }
