use failure::ResultExt;
use file::{SourceFile, SourceFileType};
use std::io::Write;
use std::path::Path;
use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio};
use std::{error, fmt, fs};

/// The resulting compiled executable.
pub struct Exe {
	path: String,
}

/// A current operating child process.
pub struct Process {
	child: Child,
}

/// A current compiling process.
pub struct CompilingProcess {
	exe: Exe,
	process: Process,
}

/// Error type for compilation.
#[derive(Debug)]
pub enum InitialisingError {
	/// Failed to initialise `cargo build`. Usually because `cargo` is not in your `PATH` or Rust is not installed.
	NoBuildCommand,
	/// Generic IO errors.
	IOError(String),
}
/// Error type for compilation.
#[derive(Debug)]
pub struct CompileError;

impl error::Error for InitialisingError {}

impl fmt::Display for InitialisingError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			InitialisingError::NoBuildCommand => {
				write!(f, "cargo build command failed to start, is rust installed?")
			}
			InitialisingError::IOError(e) => write!(f, "io error occurred. {}", e),
		}
	}
}

impl error::Error for CompileError {}

impl fmt::Display for CompileError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "compilation failed")
	}
}

impl Exe {
	/// Compile a `SourceFile` in the given directory.
	pub fn compile<P: AsRef<Path>>(
		src: &SourceFile,
		compile_dir: P,
		external_crate_name: Option<&str>,
	) -> Result<CompilingProcess, InitialisingError> {
		build_compile_dir(src, &compile_dir)?;
		fmt(&compile_dir);

		let mut exe = format!(
			"{}/target/debug/{}",
			compile_dir.as_ref().to_string_lossy(),
			src.file_name
		);
		if cfg!(windows) {
			exe.push_str(".exe");
		}

		let mut _s_tmp = String::new();
		let mut args = vec!["rustc", "--bin", "mem-code", "--", "-Awarnings"];
		if let Some(external_crate_name) = external_crate_name {
			args.push("--extern");
			_s_tmp = format!("{0}=lib{}.rlib", external_crate_name);
			args.push(&_s_tmp);
		}

		match Command::new("cargo")
			.current_dir(compile_dir)
			.args(&args)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()
		{
			Ok(c) => Ok(CompilingProcess {
				exe: Exe { path: exe },
				process: Process { child: c },
			}),
			Err(_) => Err(InitialisingError::NoBuildCommand),
		}
	}

	/// Run the `Exe`.
	pub fn run<P: AsRef<Path>>(&self, working_dir: P) -> Process {
		// lets test the external function calling
		use libloading::{Library, Symbol};
		type AddFunc = unsafe fn(isize, isize) -> isize;
		let lib =
			Library::new(Path::new(&self.path).parent().unwrap().join("memcode.dll")).unwrap();
		let value = unsafe {
			let func: Symbol<AddFunc> = lib.get(b"add").unwrap();
			func(1, 2)
		};
		dbg!(value);

		Process {
			child: Command::new(&self.path)
				.current_dir(working_dir)
				.env("RUST_BACKTRACE", "0")
				.stdout(Stdio::piped())
				.stderr(Stdio::piped())
				.spawn()
				.expect(&format!(
					"failed to start the executable {}, which is unlikely.",
					self.path
				)),
		}
	}
}

impl Process {
	/// Wait for the process to finish.
	pub fn wait(mut self) -> ExitStatus {
		self.child
			.wait()
			.expect("failed waiting for process to finish")
	}

	/// The `stderr` handle.
	pub fn stderr(&mut self) -> &mut ChildStderr {
		self.child.stderr.as_mut().expect("stderr should be piped")
	}

	/// The `stdout` handle.
	pub fn stdout(&mut self) -> &mut ChildStdout {
		self.child.stdout.as_mut().expect("stderr should be piped")
	}
}

impl CompilingProcess {
	/// Wait for the process to finish. Is successful, a `Exe` pointer will be returned, which can be run.
	pub fn wait(self) -> Result<Exe, CompileError> {
		if self.process.wait().success() {
			Ok(self.exe)
		} else {
			Err(CompileError)
		}
	}

	/// The `stderr` handle.
	pub fn stderr(&mut self) -> &mut ChildStderr {
		self.process.stderr()
	}
}

/// Constructs the compile directory with the given main source file contents.
/// Expects `SourceFileType::Rs` to define a `main()` function.
/// `SourceFileType::Rscript` will encase code in a `main()` function.
fn build_compile_dir<P: AsRef<Path>>(
	source: &SourceFile,
	compile_dir: &P,
) -> Result<(), InitialisingError> {
	let compile_dir = compile_dir.as_ref();
	let mut main_file = create_file_and_dir(&compile_dir.join("src/main.rs"))
		.map_err(|e| InitialisingError::IOError(e.to_string()))?;
	let mut cargo_file = create_file_and_dir(&compile_dir.join("Cargo.toml"))
		.map_err(|e| InitialisingError::IOError(e.to_string()))?;
	let cargo = cargotoml_contents(source);
	let content = main_contents(source);
	main_file
		.write_all(content.as_bytes())
		.context("failed writing contents of main.rs".to_string())
		.map_err(|e| InitialisingError::IOError(e.to_string()))?;
	cargo_file
		.write_all(cargo.as_bytes())
		.context("failed writing contents of Cargo.toml".to_string())
		.map_err(|e| InitialisingError::IOError(e.to_string()))?;
	Ok(())
}

/// Run `cargo fmt` in the given directory.
pub fn fmt<P: AsRef<Path>>(compile_dir: P) -> bool {
	match Command::new("cargo")
		.current_dir(compile_dir)
		.args(&["+nightly", "fmt"])
		.output()
	{
		Ok(output) => output.status.success(),
		Err(e) => {
			debug!("{}", e);
			false
		}
	}
}

fn cargotoml_contents(source: &SourceFile) -> String {
	format!(
		r#"[package]
name = "{pkg_name}"
version = "0.1.0"

[lib]
name = "memcode"
crate-type = [ "cdylib" ]
path = "src/lib.rs"

[dependencies]
{crates}
"#,
		pkg_name = source.file_name,
		crates = source
			.crates
			.iter()
			.map(|c| format!(r#"{} = "*""#, c.cargo_name))
			.collect::<Vec<_>>()
			.join("\n")
	)
}

fn main_contents(source: &SourceFile) -> String {
	format!(
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
		}
	)
}

/// Creates the specified file along with the directory to it if it doesn't exist.
fn create_file_and_dir<P: AsRef<Path>>(file: &P) -> Result<fs::File, failure::Context<String>> {
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
	use std::path;

	#[test]
	fn create_file_and_dir_test() {
		let p = path::Path::new("foo.txt");
		assert!(!p.exists());
		create_file_and_dir(&"foo.txt").unwrap();
		assert!(p.exists());
		fs::remove_file(p).unwrap();
		assert!(!p.exists());

		let p = path::Path::new("test/foo");
		assert!(!p.exists());
		create_file_and_dir(&p).unwrap();
		assert!(p.exists());
		fs::remove_file(p).unwrap();
		assert!(!p.exists());
	}

	#[test]
	fn test_build_compile_dir() {
		let source = SourceFile {
			src: TEST_CONTENTS.to_string(),
			file_type: SourceFileType::Rs,
			file_name: "test-name".to_string(),
			crates: Vec::new(),
		};

		build_compile_dir(&source, &"test/compile-dir/test-dir").unwrap();
		assert!(Path::new("test/compile-dir/test-dir/src/main.rs").exists());
		assert!(Path::new("test/compile-dir/test-dir/Cargo.toml").exists());

		fs::remove_dir_all("test/compile-dir/test-dir").unwrap();
	}

	#[test]
	fn test_run_success() {
		use std::env;
		let dir = "test/compile-dir/test-run";
		let source = SourceFile {
			src: TEST_CONTENTS.to_string(),
			file_type: SourceFileType::Rs,
			file_name: "test-name".to_string(),
			crates: Vec::new(),
		};
		assert!(Exe::compile(&source, &dir, None)
			.unwrap()
			.wait()
			.unwrap()
			.run(&env::current_dir().unwrap())
			.wait()
			.success());

		fs::remove_dir_all(dir).unwrap();
	}

	#[test]
	fn fail_compile() {
		let dir = "test/compile-dir/test-run";

		let source = SourceFile {
			src: "fn main() { let a = 1 }".to_string(),
			file_type: SourceFileType::Rs,
			file_name: "test-name".to_string(),
			crates: Vec::new(),
		};

		match Exe::compile(&source, &dir, None).unwrap().wait() {
			Err(_) => (),
			_ => panic!("expecting compilation error"),
		}

		fs::remove_dir_all(dir).unwrap();
	}

	#[test]
	fn fail_runtime() {
		use std::env;
		let dir = "test/compile-dir/test-run";

		let source = SourceFile {
			src: r#"fn main() { panic!("runtime error!"); }"#.to_string(),
			file_type: SourceFileType::Rs,
			file_name: "test-name".to_string(),
			crates: Vec::new(),
		};
		let r = Exe::compile(&source, &dir, None)
			.unwrap()
			.wait()
			.unwrap()
			.run(&env::current_dir().unwrap())
			.wait();
		assert!(!r.success());

		fs::remove_dir_all(dir).unwrap();
	}

	const TEST_CONTENTS: &str = "fn main() { println!(\"Hello, world!\"); }";
}
