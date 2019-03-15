//! Pertains to compiling a working directory into a library.

use pfh::*;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::{error, fmt, fs};

/// Constructs the compile directory.
/// Takes a list of source files and writes the contents to file.
/// Builds `Cargo.toml` using crates found in `SourceFile`.
pub fn build_compile_dir<'a, P, I>(
	compile_dir: P,
	files: I,
	linking_config: &linking::LinkingConfiguration,
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
		if let Some(cname) = linking_config.crate_name {
			if file.path == Path::new("lib.rs") {
				contents.push_str(&format!("extern crate {};\n", cname));
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
	linking_config: &linking::LinkingConfiguration,
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
	if let Some(crate_name) = linking_config.crate_name {
		args.push("--extern");
		_s_tmp = format!("{0}=lib{0}.rlib", crate_name);
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

pub fn exec<'c, P, Data, F>(
	library_file: P,
	function_name: &str,
	app_data: Data,
	std_pipes_cb: F,
) -> Result<String, &'static str>
where
	P: AsRef<Path>,
	F: Fn(&[u8]) + Send + 'static,
{
	use libloading::{Library, Symbol};

	// Has to be done to make linux builds work
	// see:
	//		https://github.com/nagisa/rust_libloading/issues/5
	//		https://github.com/nagisa/rust_libloading/issues/41
	//		https://github.com/nagisa/rust_libloading/issues/49
	//
	// Basically the api function `dlopen` will keep loaded libraries in memory to avoid
	// continuously allocating memory. It only does not release the library when thread_local data
	// is hanging around, and it seems `println!()` is something that does this.
	// Hence to avoid not having the library not updated with a new `new()` call, a different lib
	// name is passed to the function.
	// This is very annoying as it has needless fs interactions and a growing fs footprint but
	// what can you do ¯\_(ツ)_/¯
	let lib_file = rename_lib_file(library_file).map_err(|_| "failed renaming library file")?;

	// If segfaults are occurring maybe use this, SIGSEV?
	// This is shown in https://github.com/nagisa/rust_libloading/issues/41
	// let lib: Library =
	// 	libloading::os::unix::Library::open(Some(library_file.as_ref()), 0x2 | 0x1000)
	// 		.unwrap()
	// 		.into();

	let lib = Library::new(lib_file).unwrap();

	let func: Symbol<DataFunc<Data>> = unsafe { lib.get(function_name.as_bytes()).unwrap() };

	let (tx, rx) = std::sync::mpsc::channel();

	let (stdout_gag, stderr_gag) = get_gags();

	print!(""); // send an empty write to trigger non blocking reads on the gags
	eprint!("");

	let jh = std::thread::spawn(move || redirect_output(std_pipes_cb, rx, stdout_gag, stderr_gag));

	let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { func(app_data) }));

	tx.send(());
	jh.join();

	match res {
		Ok(s) => Ok(s),
		Err(_) => Err("a panic occured with evaluation"),
	}
}

/// Renames the library into a distinct file name by incrementing a counter.
/// Could fail if the number of libs grows enormous, greater than `u64`. This would mean, with
/// `u64 = 18,446,744,073,709,551,615`, even with 1KB files (prolly not) this would be
/// 18,446,744,073 TB. User will probably know something is up.
fn rename_lib_file<P: AsRef<Path>>(compiled_lib: P) -> io::Result<PathBuf> {
	let no_parent = PathBuf::new();
	let mut idx: u64 = 0;
	let parent = compiled_lib.as_ref().parent().unwrap_or(&no_parent);
	let name = |i| format!("papyrus.mem-code.lib.{}", i);
	let mut lib_path = parent.join(&name(idx));
	while lib_path.exists() {
		idx += 1;
		lib_path = parent.join(&name(idx));
	}
	std::fs::rename(&compiled_lib, &lib_path)?;
	Ok(lib_path)
}

#[cfg(windows)]
fn get_gags() -> (gag::windows::Gag<io::Stdout>, gag::windows::Gag<io::Stderr>) {
	(
		gag::windows::stdout().expect("failed to gag stdout"),
		gag::windows::stderr().expect("failed to gag stderr"),
	)
}

/// Returns (stdout, stderr).
#[cfg(unix)]
fn get_gags() -> (gag::BufferRedirect, gag::BufferRedirect) {
	(
		gag::BufferRedirect::stdout().expect("failed to gag stdout"),
		gag::BufferRedirect::stderr().expect("failed to gag stderr"),
	)
}

fn redirect_output<F, R1, R2>(
	mut cb: F,
	rx: mpsc::Receiver<()>,
	mut stdout_gag: R1,
	mut stderr_gag: R2,
) where
	F: Fn(&[u8]),
	R1: io::Read,
	R2: io::Read,
{
	use std::io::Read;

	loop {
		std::thread::sleep(std::time::Duration::from_millis(2)); // add in some delay as reading occurs to avoid smashing cpu.

		let mut buf = Vec::new();

		// read/write stderr first
		stderr_gag
			.read_to_end(&mut buf)
			.expect("failed to read stdout gag");

		stdout_gag
			.read_to_end(&mut buf)
			.expect("failed to read stdout gag");

		cb(&buf);

		match rx.try_recv() {
			Ok(_) => break,                                 // stop signal sent
			Err(mpsc::TryRecvError::Disconnected) => break, // tx dropped
			_ => (),
		};
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
	use linking::LinkingConfiguration;

	#[test]
	fn nodata_build_fmt_compile_eval_test() {
		let compile_dir = "test/nodata_build_fmt_compile_eval_test";
		let files = vec![pass_compile_eval_file()];
		let linking_config = LinkingConfiguration::default();

		// build
		build_compile_dir(&compile_dir, files.iter(), &linking_config).unwrap();
		assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
			.unwrap()
			.contains("\nlet out0 = 2+2;"));

		// // fmt
		// assert!(fmt(&compile_dir));
		// assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
		// 	.unwrap()
		// 	.contains("\n    let out0 = 2 + 2;")); // should be tabbed in (once, unless i wrap it more)

		// compile
		let path = compile(&compile_dir, &linking_config, |_| ()).unwrap();

		// eval
		let r = exec(path, "__intern_eval", (), |_| ()).unwrap(); // execute library fn

		assert_eq!(&r, "4");
	}

	#[test]
	fn brw_data_build_fmt_compile_eval_test() {
		let compile_dir = "test/brw_data_build_fmt_compile_eval_test";
		let files = vec![pass_compile_eval_file()];
		let linking_config = LinkingConfiguration::default()
			.link_external_crate(
				&compile_dir,
				"papyrus_extern_test",
				Some("test-resources/external_crate/target/debug/libexternal_crate.rlib"),
			)
			.unwrap();

		// build
		build_compile_dir(&compile_dir, files.iter(), &linking_config).unwrap();
		assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
			.unwrap()
			.contains("\nlet out0 = 2+2;"));

		// // fmt
		// assert!(fmt(&compile_dir));
		// assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
		// 	.unwrap()
		// 	.contains("\n    let out0 = 2 + 2;")); // should be tabbed in (once, unless i wrap it more)

		// compile
		let path = compile(&compile_dir, &linking_config, |_| ()).unwrap();

		// eval
		let r = exec(path, "__intern_eval", &(), |_| ()).unwrap(); // execute library fn

		assert_eq!(&r, "4");
	}

	#[test]
	fn mut_brw_data_build_fmt_compile_eval_test() {
		let compile_dir = "test/mut_brw_data_build_fmt_compile_eval_test";
		let files = vec![pass_compile_eval_file()];
		let linking_config = LinkingConfiguration::default()
			.link_external_crate(
				&compile_dir,
				"papyrus_extern_test",
				Some("test-resources/external_crate/target/debug/libexternal_crate.rlib"),
			)
			.unwrap();

		// build
		build_compile_dir(&compile_dir, files.iter(), &linking_config).unwrap();
		assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
			.unwrap()
			.contains("\nlet out0 = 2+2;"));

		// // fmt
		// assert!(fmt(&compile_dir));
		// assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
		// 	.unwrap()
		// 	.contains("\n    let out0 = 2 + 2;")); // should be tabbed in (once, unless i wrap it more)

		// compile
		let path = compile(&compile_dir, &linking_config, |_| ()).unwrap();

		// eval
		let r = exec(path, "__intern_eval", (), |_| ()).unwrap(); // execute library fn

		assert_eq!(&r, "4");
	}

	#[test]
	fn fail_compile_test() {
		let compile_dir = "test/fail_compile";
		let files = vec![faile_compile_file()];
		let linking_config = LinkingConfiguration::default();

		// build
		build_compile_dir(&compile_dir, files.iter(), &linking_config).unwrap();
		assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
			.unwrap()
			.contains("\nlet out0 = 2+;"));

		// compile
		let r = compile(&compile_dir, &linking_config, |_| ());
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
		let linking_config = LinkingConfiguration::default();

		// build
		build_compile_dir(&compile_dir, files.iter(), &linking_config).unwrap();
		assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
			.unwrap()
			.contains("\nlet out0 = panic!(\"eval panic\");"));

		// compile
		let path = compile(&compile_dir, &linking_config, |_| ()).unwrap();

		// eval
		let r = exec(&path, "__intern_eval", (), |_| ()); // execute library fn
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
