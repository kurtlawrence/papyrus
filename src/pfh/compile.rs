//! Pertains to compiling a working directory into a library.

use super::*;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, Command, ExitStatus, Stdio};
use std::{error, fmt, fs};

/// Constructs the compile directory.
/// Takes a list of source files and writes the contents to file.
/// Builds `Cargo.toml` using crates found in `SourceFile`.
pub fn build_compile_dir<'a, P, I>(
	compile_dir: P,
	files: I,
	linking_config: Option<&LinkingConfiguration>,
) -> io::Result<()>
where
	P: AsRef<Path>,
	I: Iterator<Item = &'a SourceFile>,
{
	let compile_dir = compile_dir.as_ref();

	let mut crates = Vec::new();

	// write source files
	for file in files {
		// add linked crate if there is one to lib file
		let mut contents = String::new();
		if let Some(linking_config) = linking_config {
			if file.path == Path::new("lib.rs") {
				contents.push_str(&format!("extern crate {};", linking_config.crate_name));
			}
		}
		contents.push_str(&file::code::construct(&file.contents, &file.mod_path));

		create_file_and_dir(compile_dir.join("src/").join(&file.path))?
			.write_all(contents.as_bytes())?;
		for c in file.contents.iter().flat_map(|x| &x.crates) {
			crates.push(c);
		}
	}

	// write cargo toml contents
	create_file_and_dir(compile_dir.join("Cargo.toml"))?
		.write_all(cargotoml_contents(LIBRARY_NAME, crates.into_iter()).as_bytes())?;

	Ok(())
}

pub fn compile<P, F>(
	compile_dir: P,
	linking_config: Option<&LinkingConfiguration>,
	stderr_line_cb: F,
) -> Result<PathBuf, CompilationError>
where
	P: AsRef<Path>,
	F: Fn(&str),
{
	let compile_dir = compile_dir.as_ref();
	let lib_file = compile_dir.join("target/debug/");
	let lib_file = if cfg!(windows) {
		lib_file.join(format!("{}.dll", LIBRARY_NAME))
	} else {
		lib_file.join(format!("{}", LIBRARY_NAME))
	};

	let mut _s_tmp = String::new();
	let mut args = vec!["rustc", "--", "-Awarnings"];
	if let Some(linking_config) = linking_config {
		args.push("--extern");
		_s_tmp = format!("{0}=lib{0}.rlib", linking_config.crate_name);
		args.push(&_s_tmp);
	}

	let mut child = Command::new("cargo")
		.current_dir(compile_dir)
		.args(&args)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.map_err(|_| CompilationError::NoBuildCommand)?;

	let stderr = {
		let rdr = BufReader::new(child.stderr.as_mut().expect("stderr should be piped"));
		let mut s = String::new();
		for line in rdr.lines() {
			let line = line.unwrap();
			stderr_line_cb(&line);
			s.push_str(&line);
			s.push('\n');
		}
		s
	};

	match child.wait() {
		Ok(ex) => {
			if ex.success() {
				Ok(lib_file)
			} else {
				Err(CompilationError::CompileError(stderr))
			}
		}
		Err(e) => Err(CompilationError::IOError(e)),
	}
}

pub fn execute<P: AsRef<Path>>(
	library_file: P,
	function_name: &str,
) -> Result<String, &'static str> {
	use libloading::{Library, Symbol};
	let lib = Library::new(library_file.as_ref()).unwrap();
	let res = std::panic::catch_unwind(|| unsafe {
		let func: Symbol<AddFunc> = lib.get(function_name.as_bytes()).unwrap();
		func()
	});

	match res {
		Ok(s) => Ok(s),
		Err(_) => Err("a panic occured with evaluation"),
	}
}

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
pub enum CompilationError {
	/// Failed to initialise `cargo build`. Usually because `cargo` is not in your `PATH` or Rust is not installed.
	NoBuildCommand,
	/// A compiling error occured, with the contents of the stderr.
	CompileError(String),
	/// Generic IO errors.
	IOError(io::Error),
}
/// Error type for compilation.
#[derive(Debug)]
pub struct CompileError;

impl error::Error for CompilationError {}

impl fmt::Display for CompilationError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			CompilationError::NoBuildCommand => {
				write!(f, "cargo build command failed to start, is rust installed?")
			}
			CompilationError::CompileError(e) => write!(f, "{}", e),
			CompilationError::IOError(e) => write!(f, "io error occurred: {}", e),
		}
	}
}

impl error::Error for CompileError {}

impl fmt::Display for CompileError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "compilation failed")
	}
}

// impl Exe {
// 	/// Compile a `SourceFile` in the given directory.
// 	pub fn compile<P: AsRef<Path>>(
// 		src: &SourceFile,
// 		compile_dir: P,
// 		external_crate_name: Option<&str>,
// 	) -> Result<CompilingProcess, InitialisingError> {
// 		build_compile_dir(src, &compile_dir)?;
// 		fmt(&compile_dir);

// 		let mut exe = format!(
// 			"{}/target/debug/{}",
// 			compile_dir.as_ref().to_string_lossy(),
// 			src.file_name
// 		);
// 		if cfg!(windows) {
// 			exe.push_str(".exe");
// 		}

// 		let mut _s_tmp = String::new();
// 		let mut args = vec!["rustc", "--", "-Awarnings"];
// 		if let Some(external_crate_name) = external_crate_name {
// 			args.push("--extern");
// 			_s_tmp = format!("{0}=lib{}.rlib", external_crate_name);
// 			args.push(&_s_tmp);
// 		}

// 		match Command::new("cargo")
// 			.current_dir(compile_dir)
// 			.args(&args)
// 			.stdout(Stdio::piped())
// 			.stderr(Stdio::piped())
// 			.spawn()
// 		{
// 			Ok(c) => Ok(CompilingProcess {
// 				exe: Exe { path: exe },
// 				process: Process { child: c },
// 			}),
// 			Err(_) => Err(InitialisingError::NoBuildCommand),
// 		}
// 	}

// 	/// Run the `Exe`.
// 	pub fn run(&self) -> Result<String, &'static str> {
// 		let p = Path::new(&self.path).to_path_buf();
// 		run_external_func(p)
// 	}
// }

type AddFunc = unsafe fn() -> String;
fn run_external_func(p: PathBuf) -> Result<String, &'static str> {
	unimplemented!();
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

/// Run `cargo fmt` in the given directory.
pub fn fmt<P: AsRef<Path>>(compile_dir: P) -> bool {
	match Command::new("cargo")
		.current_dir(compile_dir)
		.args(&["fmt"])
		.output()
	{
		Ok(output) => output.status.success(),
		Err(e) => {
			debug!("{}", e);
			false
		}
	}
}

/// Creates the specified file along with the directory to it if it doesn't exist.
fn create_file_and_dir<P: AsRef<Path>>(file: P) -> io::Result<fs::File> {
	let file = file.as_ref();
	debug!("trying to create file: {}", file.display());
	if let Some(parent) = file.parent() {
		fs::create_dir_all(parent)?;
	}
	fs::File::create(file)
}

#[test]
fn create_file_and_dir_test() {
	use std::path::Path;

	let p = Path::new("foo.txt");
	assert!(!p.exists());
	create_file_and_dir(&"foo.txt").unwrap();
	assert!(p.exists());
	fs::remove_file(p).unwrap();
	assert!(!p.exists());

	let p = Path::new("test/foo");
	assert!(!p.exists());
	create_file_and_dir(&p).unwrap();
	assert!(p.exists());
	fs::remove_file(p).unwrap();
	assert!(!p.exists());
}

fn cargotoml_contents<'a, I: Iterator<Item = &'a CrateType>>(lib_name: &str, crates: I) -> String {
	format!(
		r#"[package]
name = "{lib_name}"
version = "0.1.0"

[lib]
name = "{lib_name}"
crate-type = [ "cdylib" ]
path = "src/lib.rs"

[dependencies]
{crates}
"#,
		lib_name = lib_name,
		crates = crates
			.map(|c| format!(r#"{} = "*""#, c.cargo_name))
			.collect::<Vec<_>>()
			.join("\n")
	)
}
