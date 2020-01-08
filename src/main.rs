//! [![Build Status](https://github.com/kurtlawrence/papyrus/workflows/Rust%20Tests/badge.svg)](https://github.com/kurtlawrence/papyrus/actions)
//! [![Latest Version](https://img.shields.io/crates/v/papyrus.svg)](https://crates.io/crates/papyrus)
//! [![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/papyrus)
//! [![codecov](https://codecov.io/gh/kurtlawrence/papyrus/branch/master/graph/badge.svg)](https://codecov.io/gh/kurtlawrence/papyrus)
//! [![Rustc Version 1.36+](https://img.shields.io/badge/rustc-1.36+-blue.svg)](https://blog.rust-lang.org/2019/07/04/Rust-1.36.0.html)
//!
//! ## Papyrus - A rust REPL and script running tool.
//!
//! See the [rs docs](https://docs.rs/papyrus/) and the
//! [guide](https://kurtlawrence.github.io/papyrus/).
//! Look at progress and contribute on [github.](https://github.com/kurtlawrence/papyrus)
//!
//! ```sh
//! papyrus=> 2+2
//! papyrus [out0]: 4
//! ```
//!
//! Papyrus is in active development, see [changelog](https://github.com/kurtlawrence/papyrus) for updates.
//!
//! ## Overview
//!
//! ## Installation
//! Papyrus can be installed from `crates.io` or building from source on github.
//! The default installation feature set requires a `nightly` toolchain, but `stable` can be used with
//! fewer features enabled.
//!
//!
//! ## Requirements
//! Papyrus has
use papyrus::*;

fn main() {
    windows_term_hack();

    let repl = repl!();

    let app_data = &mut ();

    let run_callbacks =
        run::RunCallbacks::new(app_data).with_fmtrfn(run::fmt_based_on_terminal_width);

    let output = repl.run(run_callbacks);

    match output {
        Ok(_) => println!("Thanks for using papyrus!"),
        Err(e) => println!("papyrus crashed! {}", e),
    }
}

#[cfg(windows)]
fn windows_term_hack() {
    colored::control::set_virtual_terminal(true).ok();
}

#[cfg(not(windows))]
fn windows_term_hack() {}
