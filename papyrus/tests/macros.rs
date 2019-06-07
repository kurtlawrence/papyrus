extern crate papyrus;

use papyrus::*;

#[test]
fn macros_test() {
    // tests macro syntax

    // No external crate or data
    repl!();
    // data type
    repl!(String);
    // data type
    repl!(&String);
    // data type
    repl!(&mut String);
}

#[test]
fn different_data_patterns() {
    let repl = repl!();
    assert_eq!(repl.data.linking().data_type, None);

    let repl = repl!(String);
    assert_eq!(repl.data.linking().data_type, Some("String".to_string()));
}
