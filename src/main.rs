//! # Papyrus
//! ## A rust script running tool
//!
//! WIP

use std::io::{self, prelude::*};
use std::{fs, path};

fn main() {
	println!("Hello, world!");

	println!("command line arguments",);
	for arg in std::env::args() {
		println!("{}", arg);
	}

	match failable() {
		Err(e) => println!("{}", e),
		_ => (),
	}

	println!("Press any key to exit",);
	match io::stdin().lock().read_line(&mut String::new()) {
		_ => (),
	}
}

fn failable() -> Result<(), Box<std::error::Error>> {
	let arg_vec: Vec<String> = std::env::args().collect();

	let src_file = path::Path::new(&arg_vec[1]);
	let compile_area = "c:/papyrus-compile-area/";
	fs::create_dir_all(compile_area)?;
	let to = path::Path::new(compile_area).join("src/").join(
		src_file
			.file_name()
			.unwrap_or(path::Path::new("no_filename.rs").as_os_str()),
	);
	fs::create_dir_all(to.parent().unwrap())?;

	fs::copy(src_file, to)?;

	std::process::Command::new("cargo")
		.current_dir(compile_area)
		.arg("run")
		.spawn()?;

	Ok(())
}
