use pfh::*;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::{error, fmt, fs};

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

	drop(stderr_gag); // debugging purposes

	let jh = std::thread::spawn(move || redirect_output(std_pipes_cb, rx, stdout_gag, io::empty()));

	let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { func(app_data) }));

	tx.send(());
	jh.join();

	dbg!("should have dropped the gags by now.");

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
		dbg!("in loop");

		std::thread::sleep(std::time::Duration::from_millis(2)); // add in some delay as reading occurs to avoid smashing cpu.

		let mut buf = Vec::new();

		// read/write stderr first
		stderr_gag
			.read_to_end(&mut buf)
			.expect("failed to read stdout gag");

		dbg!("reading stdout");

		stdout_gag
			.read_to_end(&mut buf)
			.expect("failed to read stdout gag");

		dbg!(&buf);
		dbg!("invoking cb...");

		cb(&buf);

		match rx.try_recv() {
			Ok(_) => break,                                 // stop signal sent
			Err(mpsc::TryRecvError::Disconnected) => break, // tx dropped
			_ => (),
		};
	}
}
