//! ## A rust script running tool.
//!
//! See the [docs.](https://docs.rs/papyrus/0.1.2/papyrus/)
//! Look at progress and contribute on [github.](https://github.com/kurtlawrence/papyrus)
//!
//! ## WIP
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
//! Right click on a `.rs` or `.rscript` file and choose `Run with Papyrus` to compile and run code!
extern crate dirs;
extern crate failure;

mod contextmenu;

use failure::{Context, ResultExt};
use std::io::{self, BufRead, Write};
use std::path::{self, PathBuf};
use std::{fs, process};

pub use self::contextmenu::{add_right_click_menu, remove_right_click_menu};

pub enum SourceFileType {
	Rs,
	Rscript,
}

pub struct Output {
	pub status: process::ExitStatus,
}

pub struct Script {
	compile_dir: PathBuf,
	package_name: String,
}

struct CrateType {
	src_name: String,
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

		for c in get_crates(src) {
			cargo_contents.push_str(&format!("{} = \"*\"", c.cargo_name));
		}

		main_file
			.write_all(src)
			.context("failed writing contents of main.rs".to_string())?;
		cargo
			.write_all(cargo_contents.as_bytes())
			.context("failed writing contents of Cargo.toml".to_string())?;
		Ok(Script {
			package_name: package_name.to_string(),
			compile_dir: dir.to_path_buf(),
		})
	}

	/// Runs `cargo build`, then runs the `exe`  from the given directory. Stdin and Stdout are inherited (allowing live updating of progress).
	/// Waits for process to finish and returns the `Output` of the process.
	pub fn run<P: AsRef<path::Path>>(self, working_dir: &P) -> Result<Output, Context<String>> {
		let working_dir = working_dir.as_ref();
		let status = process::Command::new("cargo")
			.current_dir(&self.compile_dir)
			.arg("build")
			// .stdout(&mut stdout)
			// .stderr(&mut stderr)
			.status()
			.context("cargo command failed to start, is rust installed?".to_string())?;
		if !status.success() {
			return Err(Context::new("Build failed".to_string()));
		}

		let exe = format!(
			"{}/target/debug/{}.exe",
			self.compile_dir.clone().to_string_lossy(),
			self.package_name
		);
		let status = process::Command::new(&exe)
			.current_dir(working_dir)
			.status()
			.context(format!(
				"failed to run {} in dir {}",
				exe,
				working_dir.to_string_lossy()
			))?;

		Ok(Output {
			status: status,
			// stdout: stdout,
			// stderr: stderr,
		})
	}
}

/// Compile and run the specified source file.
/// Equivalent to calling `build_compile_dir` and then `run`.
pub fn run_from_src_file<P: AsRef<path::Path>>(src_file: P) -> Result<Output, Context<String>> {
	let src_file = src_file.as_ref();
	let filename = src_file
		.file_name()
		.map_or("papyrus-script".to_string(), |i| {
			let s = String::from(i.to_string_lossy());
			s.split('.')
				.nth(0)
				.expect("should have one element")
				.to_string()
		});
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
	let src = fs::read(src_file).context(format!("failed to read {:?}", src_file))?;

	let s = Script::build_compile_dir(&src, &filename, &dir, SourceFileType::Rs)?;
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

fn get_crates(src: &[u8]) -> Vec<CrateType> {
	let reader = io::BufReader::new(src);
	let mut crates = Vec::new();
	for line in reader.lines() {
		let line = line.expect("should be something");
		if line.contains("extern crate ") {
			match line.split(" ").nth(2) {
				Some(s) => crates.push(CrateType {
					src_name: s.replace(";", ""),
					cargo_name: s.replace(";", "").replace("_", "-"),
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
		let dir = "tests/compile-dir/test-run";
		let s = Script::build_compile_dir(
			TEST_CONTENTS.as_bytes(),
			"test-name",
			&dir,
			SourceFileType::Rs,
		).unwrap();
		s.run(&"C:/").unwrap();

		fs::remove_dir_all(dir).unwrap();
	}

	const TEST_CONTENTS: &str = "fn main() {
	println!(\"Hello, world!\");
}";
}
