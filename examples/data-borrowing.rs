#[macro_use]
extern crate papyrus;
extern crate linefeed;

use papyrus::prelude::*;

fn main() {
    // first, lets show how to pass through a simple number
    let mut v = 123;

    let mut repl = repl!(u32);

    repl = match read_until_new_line(repl, "app_data")
        .map(|eval| eval.eval(&mut v).unwrap().print())
    {
        Ok(repl) => repl,
        Err(repl) => repl,
    };
    match read_until_new_line(repl, "app_data + 123").map(|eval| eval.eval(&mut v).unwrap().print())
    {
        Ok(repl) => repl,
        Err(repl) => repl,
    };

    // second, how about a borrowed value?
    let mut v = String::from("Hello, world!");

    let mut repl = repl!(String);

    // v borrowed!
    repl = match read_until_new_line(repl, "app_data")
        .map(|eval| eval.eval(&mut v).unwrap().print())
    {
        Ok(repl) => repl,
        Err(repl) => repl,
    };
    match read_until_new_line(repl, "app_data.to_uppercase()")
        .map(|eval| eval.eval(&mut v).unwrap().print())
    {
        Ok(repl) => repl,
        Err(repl) => repl,
    };

    // TODO -- this example is broken until mutating mechnism is introduced

    // third, lets try a mutable borrow
    let mut v = String::from("Hello,");

    let mut repl = repl!(String);

    // v mutably borrowed!
    repl = match read_until_new_line(repl, "app_data")
        .map(|eval| eval.eval(&mut v).unwrap().print())
    {
        Ok(repl) => repl,
        Err(repl) => repl,
    };
    assert_eq!(&v, "Hello,");
    match read_until_new_line(repl, r#"app_data.push_str(" world!"); app_data"#)
        .map(|eval| eval.eval(&mut v).unwrap().print())
    {
        Ok(repl) => repl,
        Err(repl) => repl,
    };
    assert_eq!(&v, "Hello, world!");
}

/// Adds the newline for us!
fn read_until_new_line<T: linefeed::Terminal + 'static, D>(
    repl: Repl<repl::Read, T, D>,
    line: &str,
) -> Result<Repl<repl::Evaluate, T, D>, Repl<repl::Read, T, D>> {
    let mut repl = repl;
    for ch in line.chars().into_iter().chain("\n".chars()) {
        repl = match repl.push_input(ch) {
            papyrus::repl::PushResult::Read(r) => r,
            papyrus::repl::PushResult::Eval(r) => return Ok(r),
        }
    }

    Err(repl)
}