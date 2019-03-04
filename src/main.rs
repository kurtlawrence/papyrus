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
extern crate colored;
extern crate papyrus;
extern crate simplelog;

use papyrus::*;

fn main() {
	// turn on for logging
	// simplelog::TermLogger::init(simplelog::LevelFilter::Trace, simplelog::Config::default())
	// 	.unwrap();

	if cfg!(target_os = "windows") && !cfg!(debug_assertions) {
		// TODO fix this once colored crate is updated
		// disable colored text output on Windows as the Windows terminals do not support it yet
		colored::control::set_override(false);
	}
	let data = &mut ReplData::default().no_extern_data();
	let repl = Repl::default_terminal(data);
	repl.run();
	println!("Thanks for using papyrus!");
}
