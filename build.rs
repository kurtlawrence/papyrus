//! This build script will copy the `README.md` contents into both main.rs and lib.rs.

use std::fs;
use std::io::{BufRead, BufReader};

fn main() {
	let (lib_str, main_str) = {
		let readme = fs::read_to_string("README.md").unwrap();
		let mainrs = fs::read("src/main.rs").unwrap();
		let librs = fs::read("src/lib.rs").unwrap();
		let readme_rdr = BufReader::new(readme.trim().as_bytes());
		let mainrs_rdr = BufReader::new(&mainrs[..]);
		let librs_rdr = BufReader::new(&librs[..]);

		let mut mainrs_wtr = String::new();
		let mut librs_wtr = String::new();
		// println!("{}", String::from_utf8_lossy(&readme));

		for line in readme_rdr.lines() {
			let line = line.unwrap();
			mainrs_wtr.push_str("//! ");
			mainrs_wtr.push_str(&line);
			mainrs_wtr.push('\n');
			librs_wtr.push_str("//! ");
			librs_wtr.push_str(&line);
			librs_wtr.push('\n');
		}

		for line in mainrs_rdr.lines() {
			let line = line.unwrap();
			if !line.starts_with("//!") {
				mainrs_wtr.push_str(&line);
				mainrs_wtr.push('\n');
			}
		}
		for line in librs_rdr.lines() {
			let line = line.unwrap();
			if !line.starts_with("//!") {
				librs_wtr.push_str(&line);
				librs_wtr.push('\n');
			}
		}

		(librs_wtr, mainrs_wtr)
	};

	fs::write("src/main.rs", main_str).unwrap();
	fs::write("src/lib.rs", lib_str).unwrap();
}
