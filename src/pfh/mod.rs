//! **p**apyrus **f**ile **h**andling
//! Pertains to file operations and compilation.

pub mod code;
pub mod compile;
pub mod linking;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub use code::{CrateType, Input, SourceCode, Statement};

pub type FileMap = HashMap<PathBuf, SourceCode>;

pub const LIBRARY_NAME: &str = "papyrus_mem_code";

/// Constructs the evaluation function name given the mod sequence path.
pub fn eval_fn_name(mod_path: &[String]) -> String {
    format!("_{}_intern_eval", mod_path.join("_"))
}

pub fn into_mod_path_vec(path: &Path) -> Vec<String> {
    let mut mod_path: Vec<String> = path
        .components()
        .filter_map(|x| x.as_os_str().to_str())
        .map(|x| x.to_string())
        .collect();
    mod_path.pop(); // pops the last, which is mod.rs
    mod_path
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

#[test]
fn into_mod_path_test() {
    assert_eq!(
        into_mod_path_vec(Path::new("test/mod")),
        vec!["test".to_string()]
    );
    assert_eq!(into_mod_path_vec(Path::new("test")), Vec::<String>::new());
    assert_eq!(
        into_mod_path_vec(Path::new("test/mod/something")),
        vec!["test".to_string(), "mod".to_owned()]
    );
    assert_eq!(into_mod_path_vec(Path::new("")), Vec::<String>::new());
}
