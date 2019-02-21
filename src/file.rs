use pfh::CrateType;

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
