extern crate papyrus;

use papyrus::*;

#[test]
fn macros_test() {
    // No external crate or data
    let mut data = repl_data!();
    Repl::default_terminal(&mut data);

    // data type
    let mut data = repl_data!(String);
    Repl::default_terminal(&mut data);
}

#[test]
fn different_data_patterns() {
    let mut data = repl_data!();
    assert_eq!(data.linking().data_type, None);
    Repl::default_terminal(&mut data);

    let mut data = repl_data!(String);
    assert_eq!(data.linking().data_type, Some("String".to_string()));
    Repl::default_terminal(&mut data);

    let mut data = repl_data!(&String);
    assert_eq!(data.linking().data_type, Some("&String".to_string()));
    Repl::default_terminal(&mut data);

    let mut data = repl_data!(&mut String);
    assert_eq!(data.linking().data_type, Some("&mut String".to_string()));
    Repl::default_terminal(&mut data);
}
