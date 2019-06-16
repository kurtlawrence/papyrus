//! [![Build Status](https://travis-ci.com/kurtlawrence/papyrus.svg?branch=master)](https://travis-ci.com/kurtlawrence/papyrus)
//! [![Latest Version](https://img.shields.io/crates/v/papyrus.svg)](https://crates.io/crates/papyrus)
//! [![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/papyrus)
//! [![codecov](https://codecov.io/gh/kurtlawrence/papyrus/branch/master/graph/badge.svg)](https://codecov.io/gh/kurtlawrence/papyrus)
//! [![Rustc Version 1.30+](https://img.shields.io/badge/rustc-1.30+-blue.svg)](https://blog.rust-lang.org/2018/10/25/Rust-1.30.0.html)
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

#![warn(missing_docs)]

#[macro_use]
extern crate log;

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

/// Extendable commands for repl.
pub mod cmds;
pub mod compile;
pub mod complete;
/// Format rust code snippets using `rustfmt`.
pub mod fmt;
/// Parsing of input.
pub mod input;
/// Reading and writing output.
pub mod output;
pub mod pfh;
pub mod repl;

/// Running the repl. Requires `runnable` feature.
#[cfg(feature = "runnable")]
pub mod run;

/// Re-exports of most common types and modules.
pub mod prelude {
    pub use crate::pfh::{code, linking};
    pub use crate::repl::{self, ReadResult, Repl, ReplData, Signal};
}

pub use ::cmdtree;
#[cfg(feature = "racer-completion")]
pub use ::racer;
