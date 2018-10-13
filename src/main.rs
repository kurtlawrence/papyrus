extern crate argparse;
extern crate failure;
extern crate papyrus;

use argparse::{ArgumentParser, Store};
use failure::{Context, ResultExt};
use papyrus::*;
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
		"rc-add" => match add_right_click_menu() {
			Ok(s) => println!("added right click menu entry\n{}", s),
			Err(s) => println!("ERROR!\n{}", s),
		},
		"rc-remove" => match remove_right_click_menu() {
			Ok(s) => println!("removed right click menu entry\n{}", s),
			Err(s) => println!("ERROR!\n{}", s),
		},
		"run" => {
			match run_from_src_file(src_path) {
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
