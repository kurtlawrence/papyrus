//! This build script will copy the doc comments of `lib.rs` to `README.md`.

use std::fs;
use std::io::{BufRead, BufReader};

fn main() {
	let lib = fs::read("src/lib.rs").unwrap();
	let lib_rdr = BufReader::new(&lib[..]);

	let mut wtr = String::new();

	for line in lib_rdr.lines() {
		let line = line.unwrap();
		if line.starts_with("//!") {
			let doc = line.trim_left_matches("//!").trim();
			wtr.push_str(doc);
			wtr.push('\n');
		}
	}
	fs::write("README.md", wtr).unwrap();
}
