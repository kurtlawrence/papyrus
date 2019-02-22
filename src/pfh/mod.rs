//! **p**apyrus **f**ile **h**andling
//! Pertains to file operations and compilation.

pub mod compile;
mod file;
pub mod linking;

pub use self::file::*;
use linking::FnArgs;

pub const LIBRARY_NAME: &str = "papyrus_mem_code";

/// Constructs the evaluation function name given the mod sequence path.
pub fn eval_fn_name(mod_path: &[String]) -> String {
	format!("_{}_intern_eval", mod_path.join("_"))
}

pub fn fn_args<A>(config: Option<&linking::LinkingConfiguration<A>>) -> String {
	unimplemented!();
	match config {
		Some(c) => "".to_string(),
		None => "".to_string(),
	}
}
