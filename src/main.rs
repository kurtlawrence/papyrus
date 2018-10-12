//! # Papyrus
//! ## A rust script running tool
//!
//! ## WIP
//! Install `papyrus`.
//! `cargo install papyrus`
//!
//! Add right click context menu. (May need admin rights)
//! `papyrus rc-add`
//!
//! Remove right click context menu. (May need admin rights)
//! `papyrus rc-remove`
//!
//! Run papyrus from command line.
//! `papyrus path_to_src_file.rs` or `papyrus path_to_script_file.rscript`
//!
//! Right click on a `.rs` or `.rscript` file and choose `Run with Papyrus` to compile and run code!
extern crate argparse;

use argparse::{ArgumentParser, Store};
use std::io::{self, prelude::*};
use std::{fs, path};

fn main() {
	let mut command = String::new();
	let mut src_path = String::new();
	{
		let mut parser = ArgumentParser::new();
		parser.set_description("PAPYRUS\nA rust script runner");
		parser.refer(&mut command).add_argument(
			"command",
			Store,
			"Command argument: run, rc-add, rc-remove",
		);
		parser
			.refer(&mut src_path)
			.add_argument("src_file", Store, ".rs or .rscript source file");

		parser.parse_args_or_exit();
	}

	match command.as_str() {
		"rc-add" => match add_right_click_menu_item() {
			Ok(s) => println!("added right click menu entry\n{}", s),
			Err(s) => println!("ERROR!\n{}", s),
		},
		"rc-remove" => println!("Removing right click context menu for papyrus",),
		"run" => {
			println!("Hello, world!");

			println!("command line arguments",);
			for arg in std::env::args() {
				println!("{}", arg);
			}

			match failable() {
				Err(e) => println!("{}", e),
				_ => (),
			}

			println!("Press return to exit",);
			match io::stdin().lock().read_line(&mut String::new()) {
				_ => (),
			}
		}
		_ => println!("expecting a valid command\ntry running papyrus -h for more information",),
	}
}

trait CommandResult {
	fn convert(self) -> Result<String, String>;
}

impl CommandResult for std::io::Result<std::process::Output> {
	fn convert(self) -> Result<String, String> {
		match self {
			Ok(o) => {
				if o.status.success() {
					Ok(format!(
						"Success.\nstatus: {}\nstdout: {}\nstderr: {}",
						o.status,
						String::from_utf8_lossy(&o.stdout),
						String::from_utf8_lossy(&o.stderr)
					))
				} else {
					Err(format!(
						"ERROR!\nstatus: {}\nstdout: {}\nstderr: {}",
						o.status,
						String::from_utf8_lossy(&o.stdout),
						String::from_utf8_lossy(&o.stderr)
					))
				}
			}
			Err(e) => Err(e.to_string()),
		}
	}
}

fn add_right_click_menu_item() -> Result<String, String> {
	use std::process::Command;

	let path_to_exe = "papyrus";

	// add the .rs entry
	Command::new("reg")
		.arg("add")
		.arg("HKCR\\.rs")
		.args(&["/d", "rustsrcfile", "/f"])
		.output()
		.convert()?;
	// add the .rscript entry
	Command::new("reg")
		.arg("add")
		.arg("HKCR\\.rscript")
		.args(&["/d", "rustsrcfile", "/f"])
		.output()
		.convert()?;
	// add the shell menu
	Command::new("reg")
		.arg("add")
		.arg("HKCU\\Software\\Classes\\rustsrcfile\\shell\\Run with Papyrus\\command")
		.args(&["/d", format!("\"{}\" \"%1\"", path_to_exe).as_str(), "/f"])
		.output()
		.convert()?;

	Ok("commands successfuly executed".to_string())
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
