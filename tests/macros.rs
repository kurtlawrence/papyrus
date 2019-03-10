#[macro_use]
extern crate papyrus;

use papyrus::*;

#[test]
fn macros_test() {
    // No external crate or data
    let data = repl_data!().unwrap();
    Repl::default_terminal(data);

    let data = repl_data_brw!("crate_name", String);
    Repl::default_terminal(data);
    let data = repl_data_brw!("crate_name", "some/path/to/rlib", String);
    Repl::default_terminal(data);
    let data = repl_data_brw!("crate_name", String, "compile_dir");
    Repl::default_terminal(data);
    let data = repl_data_brw!("crate_name", "some/path/to/rlib", String, "compile_dir");
    Repl::default_terminal(data);
    let data = repl_data_brw_mut!("crate_name", String);
    Repl::default_terminal(data);
    let data = repl_data_brw_mut!("crate_name", "some/path/to/rlib", String);
    Repl::default_terminal(data);
    let data = repl_data_brw_mut!("crate_name", String, "compile_dir");
    Repl::default_terminal(data);
    let data = repl_data_brw_mut!("crate_name", "some/path/to/rlib", String, "compile_dir");
}
