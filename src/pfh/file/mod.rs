//! Pertains to source files.

pub mod code;

pub use self::code::{CrateType, Input, Item, SourceCode, Statement};
use std::path::PathBuf;

/// Holds the contents of a source file, along with meta data about itself.
pub struct SourceFile {
    /// Source code contents.
    /// Made of a vector of inputs.
    pub contents: SourceCode,
    /// Relative path to file.
    /// > relative to `lib.rs`, ie `${COMPILE_DIR}/src/path`.
    pub path: PathBuf,
    /// Relative module path.
    /// > relative to the root (lib).
    /// > sequence of module names (as if using `module::nested::name` -> `["module", "nested", "name"]`).
    pub mod_path: Vec<String>,
}

impl SourceFile {
    pub fn lib() -> Self {
        SourceFile {
            contents: Vec::new(),
            path: PathBuf::from("lib.rs"),
            mod_path: vec!["".to_string()],
        }
    }
}
