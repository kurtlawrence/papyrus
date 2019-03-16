use libloading::{Library, Symbol};
use pfh::*;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::{error, fmt, fs};

type DataFunc<D> = unsafe fn(D) -> String;

type ExecResult = Result<String, &'static str>;

pub fn exec<'c, P, Data>(library_file: P, function_name: &str, app_data: Data) -> ExecResult
where
	P: AsRef<Path>,
{
	let lib = get_lib(library_file)?;
	let func = get_func(&lib, function_name)?;

	let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { func(app_data) }));

	match res {
		Ok(s) => Ok(s),
		Err(_) => Err("a panic occured with evaluation"),
	}
}

pub fn exec_and_redirect<'c, P: AsRef<Path>, Data, W: Write + Send + 'static>(
	library_file: P,
	function_name: &str,
	app_data: Data,
	mut output_wtr: W,
) -> ExecResult {
	let lib = get_lib(library_file)?;
	let func = get_func(&lib, function_name)?;

	let (tx, rx) = std::sync::mpsc::channel();

	let (stdout_gag, stderr_gag) =
		get_gags().map_err(|_| "failed to apply redirect gags on stdout and stderr")?;

	let jh = std::thread::spawn(move || {
		redirect_output_loop(&mut output_wtr, rx, stdout_gag, stderr_gag)
	});

	let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { func(app_data) }));

	tx.send(());
	jh.join()
		.map_err(|_| "error joining redirection thread")?
		.map_err(|_| "redirection thread encountered error!")?;

	match res {
		Ok(s) => Ok(s),
		Err(_) => Err("a panic occured with evaluation"),
	}
}

fn get_lib<P: AsRef<Path>>(path: P) -> Result<Library, &'static str> {
	// If segfaults are occurring maybe use this, SIGSEV?
	// This is shown in https://github.com/nagisa/rust_libloading/issues/41
	// let lib: Library =
	// 	libloading::os::unix::Library::open(Some(library_file.as_ref()), 0x2 | 0x1000)
	// 		.unwrap()
	// 		.into();
	Library::new(path.as_ref()).map_err(|_| "failed to load library file")
}

fn get_func<'l, Data>(
	lib: &'l Library,
	name: &str,
) -> Result<Symbol<'l, DataFunc<Data>>, &'static str> {
	unsafe {
		lib.get(name.as_bytes())
			.map_err(|_| "failed to find function in library")
	}
}

fn get_gags() -> io::Result<(shh::ShhStdout, shh::ShhStderr)> {
	Ok((shh::stdout()?, shh::stderr()?))
}

fn redirect_output_loop<W: Write, R1: io::Read, R2: io::Read>(
	wtr: &mut W,
	rx: mpsc::Receiver<()>,
	mut stdout_gag: R1,
	mut stderr_gag: R2,
) -> io::Result<()> {
	loop {
		std::thread::sleep(std::time::Duration::from_millis(2)); // add in some delay as reading occurs to avoid smashing cpu.

		let mut buf = Vec::new();

		// read/write stderr first
		stderr_gag.read_to_end(&mut buf)?;

		stdout_gag.read_to_end(&mut buf)?;

		wtr.write_all(&buf)?;

		match rx.try_recv() {
			Ok(_) => break,                                 // stop signal sent
			Err(mpsc::TryRecvError::Disconnected) => break, // tx dropped
			_ => (),
		};
	}

	Ok(())
}
