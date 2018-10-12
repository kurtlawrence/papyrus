extern crate argparse;
extern crate failure;

use argparse::{ArgumentParser, Store};
use failure::{Context, ResultExt};
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

	let stdin = io::stdin();

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

			match failable(&src_path) {
				Err(e) => println!("{}", e),
				_ => (),
			}

			println!("Press return to exit",);
			match stdin.lock().read_line(&mut String::new()) {
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

	let path_to_exe = std::env::current_exe().map_err(|_| "failed to load exe path".to_string())?;

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
		.args(&[
			"/d",
			format!("{:?} \"run\" \"%1\"", path_to_exe.as_os_str()).as_str(),
			"/f",
		]).output()
		.convert()?;

	Ok("commands successfuly executed".to_string())
}

fn failable(src_file: &str) -> Result<(), Context<String>> {
	let src_file = path::Path::new(src_file);
	let compile_area = "c:/papyrus-compile-area/";
	fs::create_dir_all(compile_area).context(format!("{:?}", src_file))?;
	let to = path::Path::new(compile_area).join("src/").join(
		src_file
			.file_name()
			.unwrap_or(path::Path::new("no_filename.rs").as_os_str()),
	);
	fs::create_dir_all(to.parent().unwrap()).context(format!("{:?}", to.parent().unwrap()))?;

	fs::copy(src_file, &to).context(format!("from {:?} to {:?}", src_file, to))?;

	let mut process = std::process::Command::new("cargo")
		.current_dir(compile_area)
		.arg("run")
		.spawn()
		.context("cargo command failed to start".to_string())?;
	process.wait().context("process failed".to_string())?;

	Ok(())
}
