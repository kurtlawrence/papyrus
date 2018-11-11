use failure::ResultExt;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// The type of source file, .rs or .rscript.
pub enum SourceFileType {
	/// `*.rs` file.
	Rs,
	/// `*.rscript` file.
	Rscript,
}

/// A structure to hold the loaded file.
/// This only naively parses the file and describes the crates, file type, and contents.
pub struct SourceFile {
	/// The source code. Crates have been stripped out.
	pub src: String,
	pub file_type: SourceFileType,
	pub file_name: String,
	pub crates: Vec<CrateType>,
}

/// Some definition around crate names.
#[derive(Clone, Debug, PartialEq)]
pub struct CrateType {
	/// The source line which adds the crates.
	/// This is usually `extern crate crate_name;` or could be `extern crate crate_name as alias;`
	pub src_line: String,
	/// The name to use in cargo.
	/// Usually `crate_name` will turn into `crate-name`. The default behaviour is to replace `_` with a dash (`-`).
	pub cargo_name: String,
}

impl SourceFile {
	/// Loads a `*.rs` or `*.rscript` into a `Source`.
	pub fn load<P: AsRef<Path>>(file_path: &P) -> Result<Self, String> {
		let file_path = file_path.as_ref();
		let (filename, filetype) = {
			let f = file_path
				.file_name()
				.map_or("papyrus-script".to_string(), |i| {
					let s = String::from(i.to_string_lossy());
					s.split('.')
						.nth(0)
						.expect("should have one element")
						.to_string()
				});

			match file_path.extension() {
				Some(e) => {
					if e == "rs" {
						Ok((f, SourceFileType::Rs))
					} else if e == "rscript" {
						Ok((f, SourceFileType::Rscript))
					} else {
						Err("expecting file type *.rs or *.rscript".to_string())
					}
				}
				None => Err("expecting file type *.rs or *.rscript".to_string()),
			}
		}?;

		let src = fs::read(&file_path)
			.context(format!("failed to read {:?}", file_path))
			.map_err(|e| e.to_string())?;

		let (src, crates) = {
			// Looks through the contents and creates a collection of `CrateType`.
			// This assumes that underscores `_` will turn into dashes `-`.
			let reader = BufReader::new(&src[..]);
			let mut contents = String::new();
			let mut crates = Vec::new();
			for line in reader.lines() {
				let line = line.expect("should be something");
				if line.contains("extern crate ") {
					match line
						.split(" ")
						.nth(2)
						.map(|s| s.replace(";", "").replace("_", "-"))
					{
						Some(s) => crates.push(CrateType {
							src_line: line,
							cargo_name: s,
						}),
						None => (),
					}
				} else {
					contents.push_str(&line);
					contents.push('\n');
				}
			}

			(contents, crates)
		};

		Ok(SourceFile {
			src: src,
			file_type: filetype,
			file_name: filename,
			crates: crates,
		})
	}
}

impl CrateType {
	pub fn parse_str(string: &str) -> Result<Self, &'static str> {
		let line = match string.trim().split("\n").nth(0) {
			Some(s) => Ok(s),
			None => Err("string should have one line"),
		}?;
		if line.contains("extern crate ") {
			match line
				.split(" ")
				.nth(2)
				.map(|s| s.replace(";", "").replace("_", "-"))
			{
				Some(s) => Ok(CrateType {
					src_line: line.to_string(),
					cargo_name: s,
				}),
				None => Err("no crate name"),
			}
		} else {
			Err("line needs `extern crate `")
		}
	}
}
