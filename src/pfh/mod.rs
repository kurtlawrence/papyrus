//! **p**apyrus **f**ile **h**andling
//! Pertains to file operations and compilation.

pub mod compile;
mod file;
pub mod linking;

pub use self::file::*;

pub const LIBRARY_NAME: &str = "papyrus_mem_code";

/// Constructs the evaluation function name given the mod sequence path.
pub fn eval_fn_name(mod_path: &[String]) -> String {
	format!("_{}_intern_eval", mod_path.join("_"))
}

#[test]
fn eval_fn_name_test() {
	let path: Vec<String> = ["some", "lib", "module", "path"]
		.iter()
		.map(|x| x.to_string())
		.collect();
	assert_eq!(&eval_fn_name(&path), "_some_lib_module_path_intern_eval");
	assert_eq!(&eval_fn_name(&[]), "__intern_eval");
}
