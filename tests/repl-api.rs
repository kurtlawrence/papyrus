#[macro_use]
extern crate papyrus;

use kserd::*;
use papyrus::prelude::*;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

const BUILD_DIR_IDX: AtomicUsize = AtomicUsize::new(0);

fn unqiue_build_dir() -> PathBuf {
    format!(
        "target/testing/repl-api-{}",
        BUILD_DIR_IDX.fetch_add(1, Ordering::AcqRel)
    )
    .into()
}

fn chg_compile_dir<T, U>(mut repl: Repl<T, U>) -> Repl<T, U> {
    repl.data.with_compilation_dir(unqiue_build_dir()).unwrap();
    repl
}

#[test]
fn multiline_literal_inputs() {
    let mut repl = chg_compile_dir(repl!());

    let input = r##"let a = r#"Hello
Multiline
Input
"#;
a
"##;

    repl.line_input(input);

    // this is the same as input as we haven't .read() and got a More yet.
    assert_eq!(repl.input_buffer_line(), input);
    assert_eq!(repl.input_buffer(), input);

    match repl.read() {
        ReadResult::Read(_) => panic!("should be at Eval state!"),
        ReadResult::Eval(repl) => {
            let repl::EvalResult { repl, signal } = repl.eval(&mut ());
            assert_eq!(signal, Signal::None);
            let result_kserd = repl.print().1;
            let expected_kserd = Kserd::new_str("Hello\nMultiline\nInput\n");
            assert_eq!(result_kserd, Some((0, expected_kserd)));
        }
    }
}
