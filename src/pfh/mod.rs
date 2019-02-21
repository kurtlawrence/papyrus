//! Pertains to file operations and compilation.

pub mod compile;
mod file;
mod linking;

pub use self::file::*;
pub use self::linking::*;

pub const LIBRARY_NAME: &str = "papyrus_mem_code";

/// Constructs the evaluation function name given the mod sequence path.
pub fn eval_fn_name(mod_path: &[String]) -> String {
	format!("_{}_intern_eval", mod_path.join("_"))
}
