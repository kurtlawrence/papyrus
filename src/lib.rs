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

#![warn(missing_docs)]

#[macro_use]
extern crate log;

mod input;
mod pfh;
pub mod repl;
#[cfg(feature = "azul-widgets")]
pub mod widgets;

pub use self::pfh::linking;
pub use self::repl::{Repl, ReplData};

// re-exports
pub use cmdtree::{BuildError, Builder, BuilderChain};
