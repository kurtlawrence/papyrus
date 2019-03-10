//! Pertains to compiling a working directory into a library.

use pfh::*;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{error, fmt, fs};

/// Constructs the compile directory.
/// Takes a list of source files and writes the contents to file.
/// Builds `Cargo.toml` using crates found in `SourceFile`.
pub fn build_compile_dir<'a, P, I>(
	compile_dir: P,
	files: I,
	linking_config: Option<&linking::LinkingConfiguration>,
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
				contents.push_str(&format!("extern crate {};\n", linking_config.crate_name));
			}
		}
		contents.push_str(&file::code::construct(
			&file.contents,
			&file.mod_path,
			linking_config,
		));

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

pub fn compile<P, F>(
	compile_dir: P,
	linking_config: Option<&linking::LinkingConfiguration>,
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
		lib_file.join(format!("lib{}.so", LIBRARY_NAME))
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

type DataFunc<D> = unsafe fn(D) -> String;

pub fn exec<P, Data>(
	library_file: P,
	function_name: &str,
	app_data: Data,
) -> Result<String, &'static str>
where
	P: AsRef<Path>,
{
	use libloading::{Library, Symbol};
	let lib = Library::new(library_file.as_ref()).unwrap();
	let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
		let func: Symbol<DataFunc<Data>> = lib.get(function_name.as_bytes()).unwrap();
		func(app_data)
	}));

	match res {
		Ok(s) => Ok(s),
		Err(_) => Err("a panic occured with evaluation"),
	}
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

#[test]
fn compilation_error_fmt_test() {
	let e = CompilationError::NoBuildCommand;
	assert_eq!(
		&e.to_string(),
		"cargo build command failed to start, is rust installed?"
	);
	let e = CompilationError::CompileError("compile err".to_string());
	assert_eq!(&e.to_string(), "compile err");
	let ioe = io::Error::new(io::ErrorKind::Other, "test");
	let e = CompilationError::IOError(ioe);
	assert_eq!(&e.to_string(), "io error occurred: test");
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

#[cfg(test)]
mod tests {
	use super::*;
	use linking::{ LinkingConfiguration};

	#[test]
	fn nodata_build_fmt_compile_eval_test() {
		let compile_dir = "test/nodata_build_fmt_compile_eval_test";
		let files = vec![pass_compile_eval_file()];
		let linking_config = None;

		// build
		build_compile_dir(
			&compile_dir,
			files.iter(),
			linking_config,
		)
		.unwrap();
		assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
			.unwrap()
			.contains("\nlet out0 = 2+2;"));

		// // fmt
		// assert!(fmt(&compile_dir));
		// assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
		// 	.unwrap()
		// 	.contains("\n    let out0 = 2 + 2;")); // should be tabbed in (once, unless i wrap it more)

		// compile
		let path = compile(&compile_dir, linking_config, |_| ()).unwrap();

		// eval
		let r = exec(path, "__intern_eval", ()).unwrap(); // execute library fn

		assert_eq!(&r, "4");
	}

	#[test]
	fn brw_data_build_fmt_compile_eval_test() {
		let compile_dir = "test/brw_data_build_fmt_compile_eval_test";
		let files = vec![pass_compile_eval_file()];
		let linking_config = LinkingConfiguration::link_external_crate(
			&compile_dir,
			"papyrus_extern_test",
			Some("test-resources/external_crate/target/debug/libexternal_crate.rlib"),
		)
		.unwrap();
		let linking_config = Some(linking_config);

		// build
		build_compile_dir(
			&compile_dir,
			files.iter(),
			linking_config.as_ref() 
		)
		.unwrap();
		assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
			.unwrap()
			.contains("\nlet out0 = 2+2;"));

		// // fmt
		// assert!(fmt(&compile_dir));
		// assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
		// 	.unwrap()
		// 	.contains("\n    let out0 = 2 + 2;")); // should be tabbed in (once, unless i wrap it more)

		// compile
		let path = compile(&compile_dir, linking_config.as_ref(), |_| ()).unwrap();

		// eval
		let r = exec(path, "__intern_eval", &()).unwrap(); // execute library fn

		assert_eq!(&r, "4");
	}

	#[test]
	fn mut_brw_data_build_fmt_compile_eval_test() {
		let compile_dir = "test/mut_brw_data_build_fmt_compile_eval_test";
		let files = vec![pass_compile_eval_file()];
		let linking_config = LinkingConfiguration::link_external_crate(
			&compile_dir,
			"papyrus_extern_test",
			Some("test-resources/external_crate/target/debug/libexternal_crate.rlib"),
		)
		.unwrap();
		let linking_config = Some(linking_config);

		// build
		build_compile_dir(
			&compile_dir,
			files.iter(),
			linking_config.as_ref(),
		)
		.unwrap();
		assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
			.unwrap()
			.contains("\nlet out0 = 2+2;"));

		// // fmt
		// assert!(fmt(&compile_dir));
		// assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
		// 	.unwrap()
		// 	.contains("\n    let out0 = 2 + 2;")); // should be tabbed in (once, unless i wrap it more)

		// compile
		let path = compile(&compile_dir, linking_config.as_ref(), |_| ()).unwrap();

		// eval
		let r = exec(path, "__intern_eval", ()).unwrap(); // execute library fn

		assert_eq!(&r, "4");
	}

	#[test]
	fn fail_compile_test() {
		let compile_dir = "test/fail_compile";
		let files = vec![faile_compile_file()];
		let linking_config = None;

		// build
		build_compile_dir(
			&compile_dir,
			files.iter(),
			linking_config,
		)
		.unwrap();
		assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
			.unwrap()
			.contains("\nlet out0 = 2+;"));

		// compile
		let r = compile(&compile_dir, linking_config, |_| ());
		assert!(r.is_err());
		match r.unwrap_err() {
			CompilationError::CompileError(_) => (),
			_ => panic!("expecting CompileError"),
		}
	}

	#[test]
	fn fail_eval_test() {
		let compile_dir = "test/fail_eval_test";
		let files = vec![fail_eval_file()];
		let linking_config = None;

		// build
		build_compile_dir(
			&compile_dir,
			files.iter(),
			linking_config,
		)
		.unwrap();
		assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
			.unwrap()
			.contains("\nlet out0 = panic!(\"eval panic\");"));

		// compile
		let path = compile(&compile_dir, linking_config, |_| ()).unwrap();

		// eval
		let r = exec(&path, "__intern_eval", ()); // execute library fn
		assert!(r.is_err());
		assert_eq!(r, Err("a panic occured with evaluation"));
	}

	fn pass_compile_eval_file() -> SourceFile {
		let mut file = SourceFile::lib();
		file.contents = vec![Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "2+2".to_string(),
				semi: false,
			}],
			crates: vec![],
		}];
		file
	}

	fn faile_compile_file() -> SourceFile {
		let mut file = SourceFile::lib();
		file.contents = vec![Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "2+".to_string(),
				semi: false,
			}],
			crates: vec![],
		}];
		file
	}
	fn fail_eval_file() -> SourceFile {
		let mut file = SourceFile::lib();
		file.contents = vec![Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "panic!(\"eval panic\")".to_string(),
				semi: false,
			}],
			crates: vec![],
		}];
		file
	}
}
