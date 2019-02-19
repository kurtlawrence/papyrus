/// The type of source file, .rs or .rscript.
pub enum SourceFileType {
	/// `*.rs` file.
	Rs,
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
