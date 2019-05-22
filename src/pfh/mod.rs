//! **p**apyrus **f**ile **h**andling
//! Pertains to file operations and compilation.

pub mod code;
pub mod compile;
pub mod linking;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub use code::{CrateType, Input, SourceCode, Statement};

pub type FileMap = BTreeMap<PathBuf, SourceCode>;

pub const LIBRARY_NAME: &str = "papyrus_mem_code";

/// Constructs the evaluation function name given the mod sequence path.
/// Appends to the buffer.
pub fn eval_fn_name(mod_path: &[String], buf: &mut String) {
    buf.push('_');
    for p in mod_path {
        buf.push_str(&p);
        buf.push('_');
    }
    buf.push_str("intern_eval");	// 11 len
}

pub fn into_mod_path_vec(path: &Path) -> Vec<String> {
    // TODO make this &str rather than String
    path.components()
        .filter_map(|x| x.as_os_str().to_str())
        .map(|x| x.to_string())
        .collect()
}

#[test]
fn eval_fn_name_test() {
    let path: Vec<String> = ["some", "lib", "module", "path"]
        .iter()
        .map(|x| x.to_string())
        .collect();
    let mut s = String::new();
    eval_fn_name(&path, &mut s);
    assert_eq!(&s, "_some_lib_module_path_intern_eval");
    let mut s = String::new();
    eval_fn_name(&[], &mut s);
    assert_eq!(&s, "_intern_eval");
}

#[test]
fn into_mod_path_test() {
    assert_eq!(
        into_mod_path_vec(Path::new("test/mod")),
        vec!["test".to_string(), "mod".to_owned()]
    );
    assert_eq!(
        into_mod_path_vec(Path::new("test")),
        vec!["test".to_owned()]
    );
    assert_eq!(
        into_mod_path_vec(Path::new("test/mod/something")),
        vec!["test".to_string(), "mod".to_owned(), "something".to_owned()]
    );
    assert_eq!(into_mod_path_vec(Path::new("")), Vec::<String>::new());

    assert_eq!(
        into_mod_path_vec(Path::new("test/inner2")),
        vec!["test".to_owned(), "inner2".to_owned()]
    );
}
