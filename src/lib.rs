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

#[macro_use]
extern crate log;

extern crate colored;
extern crate dirs;
extern crate failure;
extern crate linefeed;
extern crate proc_macro2;
extern crate syn;

mod contextmenu;
mod file;
mod input;
mod repl;

use failure::{Context, ResultExt};
use file::{Source, SourceFileType};
use std::io::Write;
use std::path::{self, PathBuf};
use std::{fs, process};

pub use self::contextmenu::{add_right_click_menu, remove_right_click_menu};
pub use self::repl::Repl;
pub use self::repl::{CmdArgs, Command, Commands};

const PAPYRUS_SPLIT_PATTERN: &'static str = "<!papyrus-split>";
#[cfg(test)]
const RS_FILES: [&'static str; 2] = ["src.rs", "pwr.rs"];
#[cfg(test)]
const RSCRIPT_FILES: [&'static str; 6] = [
	"expr.rscript",
	"one.rscript",
	"expr-list.rscript",
	"count_files.rscript",
	"items.rscript",
	"dir.rscript",
];

/// A persistent structure of the script to run.
pub struct Script {
	/// The directory where `cargo build` will be run in.
	compile_dir: PathBuf,
	/// The name of the package, usually the file name.
	package_name: String,
}

impl Script {
	/// Constructs the compile directory with the given main source file contents.
	/// Expects `SourceFileType::Rs` to define a `main()` function.
	/// `SourceFileType::Rscript` will encase code in a `main()` function.
	pub fn build_compile_dir<P: AsRef<path::Path>>(
		source: &Source,
		compile_dir: &P,
	) -> Result<Self, Context<String>> {
		let dir = compile_dir.as_ref();
		let mut main_file = create_file_and_dir(&dir.join("src/main.rs"))?;
		let mut cargo = create_file_and_dir(&dir.join("Cargo.toml"))?;

		let cargo_contents = format!(
			"[package]
name = \"{pkg_name}\"
version = \"0.1.0\"

[dependencies]
{crates}
",
			pkg_name = source.file_name,
			crates = source
				.crates
				.iter()
				.map(|c| format!("{} = \"*\"", c.cargo_name))
				.collect::<Vec<_>>()
				.join("\n")
		);

		let content = format!(
			r#"
{crates}

{src}
"#,
			crates = source
				.crates
				.iter()
				.map(|c| c.src_line.clone())
				.collect::<Vec<_>>()
				.join("\n"),
			src = match source.file_type {
				SourceFileType::Rs => source.src.clone(),
				SourceFileType::Rscript => format!(
					"fn main() {{
	{}
}}",
					source.src
				),
			}
		);

		main_file
			.write_all(content.as_bytes())
			.context("failed writing contents of main.rs".to_string())?;
		cargo
			.write_all(cargo_contents.as_bytes())
			.context("failed writing contents of Cargo.toml".to_string())?;
		Ok(Script {
			package_name: source.file_name.to_string(),
			compile_dir: dir.to_path_buf(),
		})
	}

	/// Runs `cargo build`, then runs the executable  from the given directory. Stdin and Stdout are inherited (allowing live updating of progress).
	/// Waits for process to finish and returns the `Output` of the process.
	pub fn run<P: AsRef<path::Path>>(
		self,
		working_dir: &P,
	) -> Result<process::Output, Context<String>> {
		let working_dir = working_dir.as_ref();
		let status = process::Command::new("cargo")
			.current_dir(&self.compile_dir)
			.arg("build")
			.output()
			.context("cargo command failed to start, is rust installed?".to_string())?;
		if !status.status.success() {
			return Err(Context::new(format!(
				"Build failed\n{}",
				String::from_utf8_lossy(&status.stderr)
			)));
		}

		let exe = if cfg!(windows) {
			format!(
				"{}/target/debug/{}.exe",
				self.compile_dir.clone().to_string_lossy(),
				self.package_name
			)
		} else {
			format!(
				"{}/target/debug/{}",
				self.compile_dir.clone().to_string_lossy(),
				self.package_name
			)
		};

		process::Command::new(&exe)
			.current_dir(working_dir)
			.output()
			.context(format!(
				"failed to run {} in dir {}",
				exe,
				working_dir.to_string_lossy()
			))
	}
}

/// Creates the specified file along with the directory to it if it doesn't exist.
fn create_file_and_dir<P: AsRef<path::Path>>(file: &P) -> Result<fs::File, Context<String>> {
	let file = file.as_ref();
	match file.parent() {
		Some(parent) => {
			fs::create_dir_all(parent).context(format!("failed creating directory {:?}", parent))?
		}
		None => (),
	}

	fs::File::create(file).context(format!("failed creating file {:?}", file))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn create_file_and_dir_test() {
		let p = path::Path::new("foo.txt");
		assert!(!p.exists());
		create_file_and_dir(&"foo.txt").unwrap();
		assert!(p.exists());
		fs::remove_file(p).unwrap();
		assert!(!p.exists());

		let p = path::Path::new("tests/foo");
		assert!(!p.exists());
		create_file_and_dir(&p).unwrap();
		assert!(p.exists());
		fs::remove_file(p).unwrap();
		assert!(!p.exists());
	}

	#[test]
	fn test_build_compile_dir() {
		let source = Source {
			src: TEST_CONTENTS.to_string(),
			file_type: SourceFileType::Rs,
			file_name: "test-name".to_string(),
			crates: Vec::new(),
		};
		Script::build_compile_dir(&source, &"tests/compile-dir/test-dir").unwrap();
		assert!(path::Path::new("tests/compile-dir/test-dir/src/main.rs").exists());
		assert!(path::Path::new("tests/compile-dir/test-dir/Cargo.toml").exists());

		fs::remove_dir_all("tests/compile-dir/test-dir").unwrap();
	}

	#[test]
	fn test_run() {
		use std::env;
		let dir = "tests/compile-dir/test-run";
		let source = Source {
			src: TEST_CONTENTS.to_string(),
			file_type: SourceFileType::Rs,
			file_name: "test-name".to_string(),
			crates: Vec::new(),
		};
		let s = Script::build_compile_dir(&source, &dir).unwrap();
		let loc = env::current_dir().unwrap();
		println!("{:?}", loc);
		s.run(&loc).unwrap();

		fs::remove_dir_all(dir).unwrap();
	}

	const TEST_CONTENTS: &str = "fn main() {
	println!(\"Hello, world!\");
}";
}
