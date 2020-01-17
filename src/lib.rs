//! [![Build Status](https://github.com/kurtlawrence/papyrus/workflows/Rust%20Tests/badge.svg)](https://github.com/kurtlawrence/papyrus/actions)
//! [![Latest Version](https://img.shields.io/crates/v/papyrus.svg)](https://crates.io/crates/papyrus)
//! [![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/papyrus)
//! [![codecov](https://codecov.io/gh/kurtlawrence/papyrus/branch/master/graph/badge.svg)](https://codecov.io/gh/kurtlawrence/papyrus)
//! [![Rustc Version 1.39+](https://img.shields.io/badge/rustc-1.39+-blue.svg)](https://blog.rust-lang.org/2019/11/07/Rust-1.39.0.html)
//!
//! ## _Papyrus_ - A rust REPL and script running tool.
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
//! Papyrus creates a Rust REPL in your terminal. Code can be typed in, line by line with feedback on
//! the evaluation, or code can be injected via stdin handles.
//! Each code snippet is evaluated on an expression based system, so terminating with a semi-colon
//! requires more input.
//!
//! ### Example
//! ```sh
//! [lib] papyrus=> 2+2
//! papyrus [out0]: 4
//! [lib] papyrus=> println!("Hello, world!");
//! [lib] papyrus.> out0 * out0
//! Hello, world!
//! papyrus [out1]: 16
//! [lib] papyrus=> :help
//! help -- prints the help messages
//! cancel | c -- returns to the root class
//! exit -- sends the exit signal to end the interactive loop
//! Classes:
//!   edit -- Edit previous input
//!   mod -- Handle modules
//! Actions:
//!   mut -- Begin a mutable block of code
//! [lib] papyrus=> :exit
//! [lib] papyrus=> Thanks for using papyrus!
//! ```
//!
//! ## Installation
//! Papyrus can be installed from `crates.io` or building from source on github.
//! The default installation feature set requires a `nightly` toolchain, but `stable` can be used with
//! fewer features enabled.
//!
//! To install with all features:
//! ```sh
//! rustup toolchain add nightly
//! cargo +nightly install papyrus
//! ```
//!
//! To install on stable without racer completion:
//! ```sh
//! cargo +stable install papyrus --no-default-features --features="format,runnable"
//! ```
//!
//! ## Requirements
//!
//! ### Features
//! Papyrus has features sets:
//! - _format_: format code snippets using `rustfmt`
//! - _racer-completion_: enable code completion using [`racer`](https://github.com/racer-rust/racer).
//!     **Requires a nightly compiler**
//! - _runnable_: papyrus can be _run_, without needing to manually handle repl states and output
//!
//! All features are enabled by default.
//!
//! ### Cargo
//! Papyrus leverages installed binaries of both `cargo` and `rustc`. This requirement may lift in the
//! future but for now, any user wanting to use Papyrus will need an installation of Rust.

#![warn(missing_docs)]
#![deny(intra_doc_link_resolution_failure)]

#[macro_use]
extern crate log;

#[cfg(feature = "runnable")]
#[macro_use]
extern crate crossterm;

/// Build a repl instance with the default terminal.
/// If a type is specfied (ie `repl!(String)`) then the repl will be bounded to use
/// that data type. Otherwise the default `()` will be used.
#[macro_export]
macro_rules! repl {
    // Default Term, with type
    ($type:ty) => {{
        use papyrus;
        let mut r: papyrus::repl::Repl<_, $type> = papyrus::repl::Repl::default();
        r.data = unsafe { r.data.set_data_type(&format!("{}", stringify!($type))) };
        r
    }};

    // No data
    () => {{
        use papyrus;
        let r: papyrus::repl::Repl<_, ()> = papyrus::repl::Repl::default();
        r
    }};
}

#[cfg(debug_assertions)]
#[allow(unused_macros)]
macro_rules! dbg_to_file {
    ($val:expr) => {{
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("_dbg_to_file_output")
            .expect("failed to create/open _dbg_to_file_output file");
        writeln!(
            file,
            "[{}:{}] {} = {:?}",
            file!(),
            line!(),
            stringify!($val),
            $val
        )
        .unwrap();
    }};
}

/// The prefix to access commands.
const CMD_PREFIX: &str = ":";

pub mod cmds;
pub mod code;
pub mod compile;
pub mod complete;
/// Format rust code snippets using `rustfmt`.
///
/// Requires the _format_ feature.
#[cfg(feature = "format")]
pub mod fmt;
/// Parsing of input.
pub mod input;
pub mod linking;
pub mod output;
pub mod repl;

/// Running the repl. Requires `runnable` feature.
#[cfg(feature = "runnable")]
pub mod run;

/// Re-exports of most common types and modules.
pub mod prelude {
    pub use crate::repl::{self, ReadResult, Repl, ReplData, Signal};
}

pub use ::cmdtree;
#[cfg(feature = "racer-completion")]
pub use ::racer;
