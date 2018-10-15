//! # A rust script running tool.
//!
//! See the [docs.](https://docs.rs/papyrus/)
//! Look at progress and contribute on [github.](https://github.com/kurtlawrence/papyrus)
//!
//! Install `papyrus`.
//!
//! ```bash
//! cargo install papyrus
//! ```
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
//! # Implementation Notes
//!
//! - Right click on a `.rs` or `.rscript` file and choose `Run with Papyrus` to compile and run code!
//! - Papyrus will take the contents of the source code and construct a directory to be used with `cargo`. For now the directory is created under a `.papyrus` directory in the users home directory.
//! - The compiled binary will be executed with the current directory the one that houses the file. So `env::current_dir()` will return the directory of the `.rs` or `.rscript` file.
//!
//! # Example - .rs
//!
//! File `hello.rs`.
//!
//! ```text
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
//! # Example - .rscript
//!
//! File `hello.rscript`.
//!
//! ```text
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

extern crate dirs;
extern crate failure;
extern crate linefeed;
extern crate proc_macro2;
extern crate syn;

mod contextmenu;
mod input;
mod repl;

use failure::{Context, ResultExt};
use std::io::{self, BufRead, Write};
use std::path::{self, PathBuf};
use std::{fs, process};

pub use self::contextmenu::{add_right_click_menu, remove_right_click_menu};
pub use self::repl::Repl;

/// The type of source file, .rs or .rscript.
pub enum SourceFileType {
	Rs,
	Rscript,
}

/// A persistent structure of the script to run.
pub struct Script {
	/// The directory where `cargo build` will be run in.
	compile_dir: PathBuf,
	/// The name of the package, usually the file name.
	package_name: String,
}

/// Some definition around crate names.
struct CrateType {
	/// The source line which adds the crates.
	/// This is usually `extern crate crate_name;` or could be `extern crate crate_name as alias;`
	src_line: String,
	/// The name to use in cargo.
	/// Usually `crate_name` will turn into `crate-name`.
	cargo_name: String,
}

impl Script {
	/// Constructs the compile directory with the given main source file contents.
	/// Expects `SourceFileType::Rs` to define a `main()` function.
	/// `SourceFileType::Rscript` will encase code in a `main()` function.
	pub fn build_compile_dir<P: AsRef<path::Path>>(
		src: &[u8],
		package_name: &str,
		compile_dir: &P,
		src_filetype: SourceFileType,
	) -> Result<Self, Context<String>> {
		let dir = compile_dir.as_ref();
		let mut main_file = create_file_and_dir(&dir.join("src/main.rs"))?;
		let mut cargo = create_file_and_dir(&dir.join("Cargo.toml"))?;

		let mut cargo_contents = format!(
			"[package]
name = \"{}\"
version = \"0.1.0\"

[dependencies]
",
			package_name
		);

		let crates = get_crates(src);
		for c in crates.iter() {
			cargo_contents.push_str(&format!("{} = \"*\"", c.cargo_name));
		}

		let content = match src_filetype {
			SourceFileType::Rs => src.iter().map(|x| *x).collect(),
			SourceFileType::Rscript => {
				let reader = io::BufReader::new(src);
				let mut ret = Vec::with_capacity(src.len());

				for c in crates {
					ret.append(&mut c.src_line.into_bytes());
					"\n".as_bytes().iter().for_each(|b| ret.push(*b));
				}

				"fn main() {\n".as_bytes().iter().for_each(|b| ret.push(*b));
				for line in reader.lines() {
					let line = line.expect("should be something");
					if !line.contains("extern crate ") {
						"\t".as_bytes().iter().for_each(|b| ret.push(*b));
						ret.append(&mut line.into_bytes());
						"\n".as_bytes().iter().for_each(|b| ret.push(*b));
					}
				}
				"}".as_bytes().iter().for_each(|b| ret.push(*b));

				ret
			}
		};

		main_file
			.write_all(&content)
			.context("failed writing contents of main.rs".to_string())?;
		cargo
			.write_all(cargo_contents.as_bytes())
			.context("failed writing contents of Cargo.toml".to_string())?;
		Ok(Script {
			package_name: package_name.to_string(),
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

/// Compile and run the specified source file.
/// Equivalent to calling `Script` `build_compile_dir` and then `run`.
pub fn run_from_src_file<P: AsRef<path::Path>>(
	src_file: P,
) -> Result<process::Output, Context<String>> {
	let src_file = src_file.as_ref().canonicalize().context(format!(
		"failed to canonicalize src_file {}",
		src_file.as_ref().clone().to_string_lossy()
	))?;
	let (filename, filetype) = {
		let f = src_file
			.file_name()
			.map_or("papyrus-script".to_string(), |i| {
				let s = String::from(i.to_string_lossy());
				s.split('.')
					.nth(0)
					.expect("should have one element")
					.to_string()
			});

		match src_file.extension() {
			Some(e) => if e == "rs" {
				Ok((f, SourceFileType::Rs))
			} else if e == "rscript" {
				Ok((f, SourceFileType::Rscript))
			} else {
				Err(Context::new(
					"expecting file type *.rs or *.rscript".to_string(),
				))
			},
			None => Err(Context::new(
				"expecting file type *.rs or *.rscript".to_string(),
			)),
		}
	}?;
	let dir = dirs::home_dir().ok_or(Context::new("no home directory".to_string()))?;
	let mut dir = path::PathBuf::from(format!("{}/.papyrus", dir.to_string_lossy()));
	src_file.components().for_each(|c| {
		if let path::Component::Normal(c) = c {
			let s = String::from(c.to_string_lossy());
			if s.contains(".") {
				dir.push(s.split('.').nth(0).expect("should have one element"))
			} else {
				dir.push(s)
			}
		}
	});
	let src = fs::read(&src_file).context(format!("failed to read {:?}", src_file))?;

	let s = Script::build_compile_dir(&src, &filename, &dir, filetype)?;
	s.run(&src_file.parent().unwrap())
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

/// Looks through the contents and creates a collection of `CrateType`.
/// This assumes that underscores `_` will turn into dashes `-`.
fn get_crates(src: &[u8]) -> Vec<CrateType> {
	let reader = io::BufReader::new(src);
	let mut crates = Vec::new();
	for line in reader.lines() {
		let line = line.expect("should be something");
		if line.contains("extern crate ") {
			match line
				.split(" ")
				.nth(2)
				.map(|s| s.replace(";", "").replace("_", "-"))
			{
				Some(s) => crates.push(CrateType {
					src_line: line,
					cargo_name: s,
				}),
				None => (),
			}
		}
	}

	crates
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
		Script::build_compile_dir(
			TEST_CONTENTS.as_bytes(),
			"test-name",
			&"tests/compile-dir/test-dir",
			SourceFileType::Rs,
		).unwrap();
		assert!(path::Path::new("tests/compile-dir/test-dir/src/main.rs").exists());
		assert!(path::Path::new("tests/compile-dir/test-dir/Cargo.toml").exists());

		fs::remove_dir_all("tests/compile-dir/test-dir").unwrap();
	}

	#[test]
	fn test_run() {
		use std::env;
		let dir = "tests/compile-dir/test-run";
		let s = Script::build_compile_dir(
			TEST_CONTENTS.as_bytes(),
			"test-name",
			&dir,
			SourceFileType::Rs,
		).unwrap();
		let loc = env::current_dir().unwrap();
		println!("{:?}", loc);
		s.run(&loc).unwrap();

		fs::remove_dir_all(dir).unwrap();
	}

	const TEST_CONTENTS: &str = "fn main() {
	println!(\"Hello, world!\");
}";
}
