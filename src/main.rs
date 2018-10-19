//! # papyrus
//!
//! [![Build Status](https://travis-ci.com/kurtlawrence/papyrus.svg?branch=master)](https://travis-ci.com/kurtlawrence/papyrus) [![Latest Version](https://img.shields.io/crates/v/papyrus.svg)](https://crates.io/crates/papyrus) [![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/papyrus)
//!
//! ## A rust REPL and script running tool
//!
//! See the [docs.](https://docs.rs/papyrus/)
//! Look at progress and contribute on [github.](https://github.com/kurtlawrence/papyrus)
//!
//! ## Installation
//!
//! `papyrus` depends on `proc-macro2` and `syn` which contains features that are only available on a nightly compiler. Further to this, the features are underneath a config flag, so compiling requires the `RUSTFLAGS` environment variable to include `--cfg procmacro2_semver_exempt`.
//!
//! Linux, Mac
//!
//! ```bash
//! RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install papyrus
//! ```
//!
//! Windows
//!
//! ```bash
//! $env:RUSTFLAGS="--cfg procmacro2_semver_exempt"
//! cargo install papyrus;
//! ```
//!
//! ## REPL
//!
//! `papyrus run` will start the repl!
//!
//! ## Shell Context Menu
//!
//! Add right click context menu. (May need admin rights)
//!
//! ```bash
//! papyrus rc-add
//! ```
//!
//! Remove right click context menu. (May need admin rights)
//!
//! ```bash
//! papyrus rc-remove
//! ```
//!
//! Run papyrus from command line.
//!
//! ```bash
//! papyrus run path_to_src_file.rs
//! papyrus run path_to_script_file.rscript
//! ```
//!
//! ## Implementation Notes
//!
//! - Right click on a `.rs` or `.rscript` file and choose `Run with Papyrus` to compile and run code!
//! - Papyrus will take the contents of the source code and construct a directory to be used with `cargo`. For now the directory is created under a `.papyrus` directory in the users home directory.
//! - The compiled binary will be executed with the current directory the one that houses the file. So `env::current_dir()` will return the directory of the `.rs` or `.rscript` file.
//!
//! ## Example - .rs
//!
//! File `hello.rs`.
//!
//! ```sh
//! extern crate some_crate;
//!
//! fn main() {
//!   println!("Hello, world!");
//! }
//! ```
//!
//! Use papyrus to execute code.
//!
//! ```bash
//! papyrus run hello.rs
//! ```
//!
//! The `src/main.rs` will be populated with the same contents as `hello.rs`. A `Cargo.toml` file will be created, where `some_crate` will be added as a dependency `some-crate = "*"`.
//!
//! ## Example - .rscript
//!
//! File `hello.rscript`.
//!
//! ```sh
//! extern crate some_crate;
//!
//! println!("Hello, world!");
//! ```
//!
//! Use papyrus to execute code.
//!
//! ```bash
//! papyrus run hello.rscript
//! ```
//!
//! The `src/main.rs` will be populated with a main function encapsulating the code, and crate references placed above it. A similar `Cargo.toml` will be created as before.
extern crate argparse;
extern crate papyrus;
extern crate simplelog;

use argparse::{ArgumentParser, Store};
use papyrus::*;
use std::io::{self, prelude::*};

fn main() {
	if cfg!(debug) {
		simplelog::TermLogger::init(simplelog::LevelFilter::Trace, simplelog::Config::default())
			.unwrap();
	}
	let repl = Repl::new();
	repl.run();

	let mut command = String::new();
	let mut src_path = String::new();
	{
		let mut parser = ArgumentParser::new();
		parser.set_description("PAPYRUS\nA rust script runner");
		parser.refer(&mut command).add_argument(
			"command",
			Store,
			"Command argument: run, rc-add, rc-remove",
		);
		parser
			.refer(&mut src_path)
			.add_argument("src_file", Store, ".rs or .rscript source file");

		parser.parse_args_or_exit();
	}

	let stdin = io::stdin();

	match command.as_str() {
		"rc-add" => match add_right_click_menu() {
			Ok(s) => println!("added right click menu entry\n{}", s),
			Err(s) => println!("ERROR!\n{}", s),
		},
		"rc-remove" => match remove_right_click_menu() {
			Ok(s) => println!("removed right click menu entry\n{}", s),
			Err(s) => println!("ERROR!\n{}", s),
		},
		"run" => {
			match run_from_src_file(src_path) {
				Err(e) => println!("{}", e),
				_ => (),
			}

			println!("Press return to exit",);
			match stdin.lock().read_line(&mut String::new()) {
				_ => (),
			}
		}
		_ => println!("expecting a valid command\ntry running papyrus -h for more information",),
	}
}
