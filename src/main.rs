//! # papyrus
//! 
//! [![Build Status](https://travis-ci.com/kurtlawrence/papyrus.svg?branch=master)](https://travis-ci.com/kurtlawrence/papyrus) [![Latest Version](https://img.shields.io/crates/v/papyrus.svg)](https://crates.io/crates/papyrus) [![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/papyrus) [![codecov](https://codecov.io/gh/kurtlawrence/papyrus/branch/master/graph/badge.svg)](https://codecov.io/gh/kurtlawrence/papyrus)
//! [![Rustc Version 1.30+](https://img.shields.io/badge/rustc-1.30+-blue.svg)](https://blog.rust-lang.org/2018/10/25/Rust-1.30.0.html)
//! 
//! ## A rust REPL and script running tool
//! 
//! See the [rs docs](https://docs.rs/papyrus/) and the [usage guide](https://kurtlawrence.github.io/papyrus/)
//! Look at progress and contribute on [github.](https://github.com/kurtlawrence/papyrus)
//! 
//! ## Installation
//! 
//! ```bash
//! cargo install papyrus
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
extern crate colored;
extern crate papyrus;
extern crate simplelog;

use argparse::{ArgumentParser, Store};
use papyrus::*;

fn main() {
	// turn on for logging
	// simplelog::TermLogger::init(simplelog::LevelFilter::Trace, simplelog::Config::default())
	// 	.unwrap();

	if cfg!(target_os = "windows") {
		// disable colored text output on Windows as the Windows terminals do not support it yet
		colored::control::set_override(false);
	}

	let mut command = String::new();
	let mut src_path = String::new();
	{
		let mut parser = ArgumentParser::new();
		parser.set_description("PAPYRUS\nA rust repl and script runner");
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

	match command.as_str() {
		"rc-add" => match add_right_click_menu() {
			Ok(s) => println!("added right click menu entry\n{}", s),
			Err(s) => println!("ERROR!\n{}", s),
		},
		"rc-remove" => match remove_right_click_menu() {
			Ok(s) => println!("removed right click menu entry\n{}", s),
			Err(s) => println!("ERROR!\n{}", s),
		},
		"run" | "" => {
			let repl = if !src_path.is_empty() {
				Repl::with_file(&src_path)
			} else {
				Repl::new()
			};
			repl.run();
			println!("Thanks for using papyrus!");
		}
		_ => println!("expecting a valid command\ntry running papyrus -h for more information",),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::env;
	use std::path::PathBuf;
	use std::process;

	fn exe() -> PathBuf {
		let mut loc = env::current_exe().unwrap().canonicalize().unwrap();
		loc.pop();
		loc.pop();
		if cfg!(windows) {
			loc.join("papyrus.exe")
		} else {
			loc.join("papyrus")
		}
	}

	#[test]
	fn run_rc_add() {
		let exe = exe();
		println!("{}", exe.to_string_lossy());
		process::Command::new(exe).arg("rc-add").spawn().unwrap();
	}

	#[test]
	fn run_rc_remove() {
		let exe = exe();
		println!("{}", exe.to_string_lossy());
		process::Command::new(exe).arg("rc-remove").spawn().unwrap();
	}

	#[test]
	fn run_repl() {
		let exe = exe();
		println!("{}", exe.to_string_lossy());
		let mut p1 = process::Command::new(&exe).spawn().unwrap();
		let mut p2 = process::Command::new(&exe).arg("run").spawn().unwrap();

		std::thread::sleep(std::time::Duration::from_millis(500));

		p1.kill().unwrap();
		p2.kill().unwrap();
	}

	#[test]
	fn fail_cmd() {
		let exe = exe();
		println!("{}", exe.to_string_lossy());
		let out = process::Command::new(exe).arg("adf").output().unwrap();
		assert_eq!(
			String::from_utf8_lossy(&out.stdout),
			"expecting a valid command\ntry running papyrus -h for more information\n".to_string()
		);
	}
}
