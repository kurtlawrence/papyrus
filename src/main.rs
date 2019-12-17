//! [![Build Status](https://travis-ci.com/kurtlawrence/papyrus.svg?branch=master)](https://travis-ci.com/kurtlawrence/papyrus)
//! [![Latest Version](https://img.shields.io/crates/v/papyrus.svg)](https://crates.io/crates/papyrus)
//! [![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/papyrus)
//! [![codecov](https://codecov.io/gh/kurtlawrence/papyrus/branch/master/graph/badge.svg)](https://codecov.io/gh/kurtlawrence/papyrus)
//! [![Rustc Version 1.35+](https://img.shields.io/badge/rustc-1.35+-blue.svg)](https://blog.rust-lang.org/2018/10/25/Rust-1.30.0.html)
//!
//! A rust REPL and script running tool.
//!
//! See the [rs docs](https://docs.rs/papyrus/).
//! Look at progress and contribute on [github.](https://github.com/kurtlawrence/papyrus)
//!
//! ```sh
//! papyrus=> 2+2
//! papyrus [out0]: 4
//! ```
//!
//! Papyrus is in active development, see [changelog](https://github.com/kurtlawrence/papyrus) for updates
use papyrus::*;

fn main() {
    windows_term_hack();

    let repl = repl!();

    let output = repl.run(&mut ());

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
