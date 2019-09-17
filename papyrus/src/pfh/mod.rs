//! **p**apyrus **f**ile **h**andling
//! Pertains to file operations and compilation.

use crate::code::SourceCode;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Mapping of modules to source code.
pub type ModsMap = BTreeMap<PathBuf, SourceCode>;

/// Constructs the evaluation function name given the mod sequence path.
/// Appends to the buffer.
pub fn eval_fn_name<S: AsRef<str>>(mod_path: &[S], buf: &mut String) {
    buf.push('_');
    for p in mod_path {
        buf.push_str(p.as_ref());
        buf.push('_');
    }
    buf.push_str("intern_eval"); // 11 len
}

/// Calculates the length of the evaluation function name.
/// Used for performance.
pub fn eval_fn_name_length<S: AsRef<str>>(mod_path: &[S]) -> usize {
    12 + mod_path.iter().map(|x| x.as_ref().len() + 1).sum::<usize>()
}

/// Transforms a path into a vector of components.
pub fn into_mod_path_vec(path: &Path) -> Vec<&str> {
    path.iter().filter_map(|x| x.to_str()).collect()
}

#[test]
fn eval_fn_name_test() {
    let path: Vec<String> = ["some", "lib", "module", "path"]
        .iter()
        .map(|x| x.to_string())
        .collect();
    let mut s = String::new();
    eval_fn_name(&path, &mut s);

    let ans = "_some_lib_module_path_intern_eval";
    assert_eq!(&s, ans);
    assert_eq!(eval_fn_name_length(&path), ans.len());

    let mut s = String::new();
    eval_fn_name::<&str>(&[], &mut s);

    let ans = "_intern_eval";
    assert_eq!(&s, ans);
    assert_eq!(eval_fn_name_length::<&str>(&[]), ans.len());
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
