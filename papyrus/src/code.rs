//! Source file and crate contents.
//!
//! Input is parsed as Rust code using the `syn` crate. `papyrus` does not differentiate the
//! myriad of classications for the input, rather it categorises them into [`Item`]s, [`Statement`]s,
//! and [`CrateType`]s.
//!
//! `papyrus` will parse a string input into a [`Input`], and these aggregate into a [`SourceCode`]
//! structure, which flattens each input.
//!
//! # Examples
//!
//! Building some source code.
//! ```rust
//! use papyrus::code::*;
//!
//! let mut src = SourceCode::new();
//! src.stmts.push(StmtGrp(vec![Statement {
//! 	expr: String::from("let a = 1"),
//! 	semi: true
//!     },
//!     Statement {
//! 	expr: String::from("a"),
//! 	semi: false
//!     }
//! ]));
//! ```
//!
//! Crates have some more structure around them.
//! ```rust
//! use papyrus::code::*;
//!
//! let input = "extern crate a_crate as acrate;";
//! let cr = CrateType::parse_str(input).unwrap();
//!
//! assert_eq!(&cr.src_line, input);
//! assert_eq!(&cr.cargo_name, "a-crate");
//! ```
//!
//! [`CrateType`]: CrateType
//! [`Input`]: Input
//! [`Item`]: Item
//! [`SourceCode`]: SourceCode
//! [`Statement`]: Statement
use super::*;
use crate::linking::LinkingConfiguration;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

type ReturnRange = std::ops::Range<usize>;
type ReturnRangeMap<'a> = fxhash::FxHashMap<&'a Path, ReturnRange>;

/// Mapping of modules to source code.
pub type ModsMap = BTreeMap<PathBuf, SourceCode>;

/// An input collection
#[derive(Debug, PartialEq, Clone)]
pub struct Input {
    /// Module-level items (`fn`, `enum`, `type`, `struct`, etc.)
    pub items: Vec<Item>,
    /// Inner statements and declarations.
    pub stmts: Vec<Statement>,
    /// The referenced crates.
    pub crates: Vec<CrateType>,
}

/// The flattened representation of source code.
/// Statements are grouped based on the the 'out' number.
#[derive(Clone)]
pub struct SourceCode {
    /// Module-level items (`fn`, `enum`, `type`, `struct`, etc.)
    pub items: Vec<Item>,
    /// Inner statements and declarations.
    pub stmts: Vec<StmtGrp>,
    /// The referenced crates.
    pub crates: Vec<CrateType>,
}

impl SourceCode {
    /// Construct new `SourceCode`.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            stmts: Vec::new(),
            crates: Vec::new(),
        }
    }
}

/// Group of statements that result in an expression to evaulate.
///
/// # Example
/// ```rust
/// # use papyrus::code::*;
/// let stmt1 = Statement { expr: "let a = 1".to_string(), semi: true };
/// let stmt2 = Statement { expr: "a".to_string(), semi: false };
/// let grp = StmtGrp(vec![stmt1, stmt2]);
/// assert_eq!(&grp.src_line(), "let a = 1; a");
/// ```
#[derive(Clone)]
pub struct StmtGrp(pub Vec<Statement>);

impl StmtGrp {
    /// The statements as a single line of rust code.
    pub fn src_line(&self) -> String {
        let mut buf = String::with_capacity(self.assign_let_binding_length(0));

        let stmts = &self.0;

        for stmt in stmts {
            buf.push_str(&stmt.expr);
            if stmt.semi {
                buf.push(';');
            }
            buf.push(' ');
        }

        buf.pop();

        buf
    }

    /// Stringfy's the statements and assigns trailing expressions with `let out# = expr;`.
    fn assign_let_binding(&self, input_num: usize, buf: &mut String) {
        let stmts = &self.0;

        for stmt in &stmts[0..stmts.len().saturating_sub(1)] {
            buf.push_str(&stmt.expr);
            if stmt.semi {
                buf.push(';');
            }
            buf.push('\n');
        }

        if stmts.len() > 0 {
            buf.push_str("let out");
            buf.push_str(&input_num.to_string());
            buf.push_str(" = ");
            buf.push_str(&stmts[stmts.len() - 1].expr);
            buf.push(';');
        }
    }

    fn assign_let_binding_length(&self, input_num: usize) -> usize {
        let stmts = &self.0;
        let mut cap = 0;

        for stmt in &stmts[0..stmts.len().saturating_sub(1)] {
            cap += 1 + stmt.expr.len();
            if stmt.semi {
                cap += 1;
            }
        }

        cap += if stmts.len() > 0 {
            7 + input_num.to_string().len() + 3 + stmts[stmts.len() - 1].expr.len() + 1
        } else {
            0
        };

        cap
    }
}

/// Construct a single string containing all the source code in `mods_map`.
pub fn construct_source_code<'a>(
    mods_map: &'a ModsMap,
    linking_config: &LinkingConfiguration,
) -> (String, ReturnRangeMap<'a>) {
    // assumed to be sorted, FileMap is BTreeMap

    let (cap, map) = calc_capacity(mods_map, linking_config);

    let mut contents = String::with_capacity(cap);

    // add in external crates
    for external in linking_config.external_libs.iter() {
        external.construct_code_str(&mut contents);
    }

    // do the lib first
    if let Some(lib) = mods_map.get(Path::new("lib")) {
        code::append_buffer(
            lib,
            &into_mod_path_vec(Path::new("lib")),
            linking_config,
            &mut contents,
        );
    }

    for (prev_lvl, new_lvl, file, src_code) in mods_map_with_lvls(mods_map) {
        match new_lvl.cmp(&prev_lvl) {
            Ordering::Equal | Ordering::Less => {
                // need to close off the open modules
                let diff = prev_lvl - new_lvl; // should always be >= 0
                for _ in 0..=diff {
                    contents.push('}');
                }
                contents.push('\n');
            }
            _ => (),
        }

        contents.push_str("mod ");
        contents.push_str(
            file.iter()
                .last()
                .and_then(|x| x.to_str())
                .expect("should convert fine"),
        );
        contents.push_str(" {\n");
        code::append_buffer(
            src_code,
            &into_mod_path_vec(file),
            linking_config,
            &mut contents,
        );
    }

    // close off any outstanding modules
    let lvl = mods_map_with_lvls(mods_map)
        .last()
        .map(|x| x.1)
        .unwrap_or(0);
    for _ in 0..lvl {
        contents.push('}');
    }

    debug_assert_eq!(
        cap,
        contents.len(),
        "failed at calculating the correct capacity"
    );

    (contents, map)
}

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
fn eval_fn_name_length<S: AsRef<str>>(mod_path: &[S]) -> usize {
    12 + mod_path.iter().map(|x| x.as_ref().len() + 1).sum::<usize>()
}

/// Transforms a path into a vector of components.
pub fn into_mod_path_vec(path: &Path) -> Vec<&str> {
    path.iter().filter_map(|x| x.to_str()).collect()
}

/// **Skips lib**
fn mods_map_with_lvls(
    mods_map: &ModsMap,
) -> impl Iterator<Item = (usize, usize, &Path, &SourceCode)> {
    let mut prev = 0;
    mods_map
        .iter()
        .filter(|x| x.0 != Path::new("lib"))
        .map(move |x| {
            let c = x.0.iter().count();
            let r = (prev, c, x.0.as_path(), x.1);
            prev = c;
            r
        })
}

fn calc_capacity<'a>(
    mods_map: &'a ModsMap,
    linking_config: &LinkingConfiguration,
) -> (usize, ReturnRangeMap<'a>) {
    fn mv_rng(mut rng: ReturnRange, by: usize) -> ReturnRange {
        rng.start += by;
        rng.end += by;
        rng
    }

    let mut cap = 0;

    let mut map =
        HashMap::with_capacity_and_hasher(mods_map.len(), fxhash::FxBuildHasher::default());

    for external in linking_config.external_libs.iter() {
        cap += external.construct_code_str_length();
    }

    // do the lib first
    if let Some(lib) = mods_map.get(Path::new("lib")) {
        let (src_code_len, src_code_return) =
            append_buffer_length(lib, &into_mod_path_vec(Path::new("lib")), linking_config);

        map.insert(Path::new("lib"), mv_rng(src_code_return, cap));

        cap += src_code_len;
    }

    for (prev_lvl, new_lvl, file, src_code) in mods_map_with_lvls(mods_map) {
        match new_lvl.cmp(&prev_lvl) {
            Ordering::Equal | Ordering::Less => {
                cap += prev_lvl - new_lvl + 2;
            }
            _ => (),
        }

        cap += 4; // mod
        cap += file
            .iter()
            .last()
            .and_then(|x| x.to_str())
            .map(|x| x.len())
            .unwrap_or(0);
        cap += 3; // }\n

        let (src_code_len, src_code_return) =
            append_buffer_length(src_code, &into_mod_path_vec(file), linking_config);

        map.insert(file, mv_rng(src_code_return, cap));

        cap += src_code_len;
    }

    // close off any outstanding modules
    let lvl = mods_map_with_lvls(mods_map)
        .last()
        .map(|x| x.1)
        .unwrap_or(0);
    cap += lvl;

    (cap, map)
}

/// Build the buffer with the stringified contents of SourceCode
fn append_buffer<S: AsRef<str>>(
    src_code: &SourceCode,
    mod_path: &[S],
    linking_config: &linking::LinkingConfiguration,
    buf: &mut String,
) {
    // wrap stmts
    buf.push_str("#[no_mangle]\npub extern \"C\" fn "); // 31 len
    eval_fn_name(mod_path, buf);
    buf.push('(');
    linking_config.construct_fn_args(buf);
    buf.push_str(") -> kserd::Kserd<'static> {\n"); // 29 len

    // add stmts
    let c = src_code.stmts.len();
    if c >= 1 {
        // only add statements if more than zero!
        src_code.stmts.iter().enumerate().for_each(|(i, x)| {
            x.assign_let_binding(i, buf);
            buf.push('\n');
        });
        buf.push_str("kserd::ToKserd::into_kserd(out");
        buf.push_str(&c.saturating_sub(1).to_string());
        buf.push_str(").unwrap().to_owned()\n");
    } else {
        buf.push_str("kserd::Kserd::new_str(\"no statements\")\n");
    }
    buf.push_str("}\n");

    // add items
    for item in src_code.items.iter() {
        buf.push_str(item.as_str());
        buf.push('\n');
    }
}

fn append_buffer_length<S: AsRef<str>>(
    src_code: &SourceCode,
    mod_path: &[S],
    linking_config: &linking::LinkingConfiguration,
) -> (usize, ReturnRange) {
    // wrap stmts
    let mut cap =
        31 + eval_fn_name_length(mod_path) + 1 + linking_config.construct_fn_args_length() + 29;

    // add stmts
    let c = src_code.stmts.len();
    let (add, rng) = if c >= 1 {
        let stmts = src_code
            .stmts
            .iter()
            .enumerate()
            .map(|(i, x)| x.assign_let_binding_length(i) + 1)
            .sum::<usize>();
        let return_str = 30 // kserd::ToKserd::into_kserd(out
            + c.saturating_sub(1).to_string().len()
            + 22; // ).unwrap().to_owned()\n

        (
            stmts + return_str,
            cap + stmts..cap + stmts + return_str - 1,
        )
    } else {
        // kserd::Kserd::new_str("no statements")\n
        (39, cap..cap + 38)
    };
    cap += add + 2; // }\n

    // add items
    for item in src_code.items.iter() {
        cap += item.len() + 1;
    }

    (cap, rng)
}

/// A single item.
pub type Item = String;

/// Represents an inner statement.
#[derive(Debug, PartialEq, Clone)]
pub struct Statement {
    /// The code, not including the trailing semi if there is one.
    pub expr: String,
    /// Flags whether there is a trailing semi.
    pub semi: bool,
}

/// Some definition around crate names.
///
/// Crates are parsed and made suitable for `Cargo.toml`. The input line is kept verbatim.
///
/// # Examples
/// ```rust
/// # use papyrus::code::CrateType;
/// let input = "extern crate a_crate as acrate;";
/// let cr = CrateType::parse_str(input).unwrap();
/// assert_eq!(&cr.src_line, input);
/// assert_eq!(&cr.cargo_name, "a-crate");
/// ```
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
    /// Parses a string to return the `CrateType`.
    pub fn parse_str(string: &str) -> Result<Self, &'static str> {
        let line = string
            .replace(";", "")
            .replace("_", "-")
            .trim()
            .split("\n")
            .nth(0)
            .expect("string should have one line")
            .to_string();
        if line.contains("extern crate ") {
            Ok(CrateType {
                src_line: string.to_string(),
                cargo_name: line
                    .split(" ")
                    .nth(2)
                    .expect("should always have trailing item")
                    .to_string(),
            })
        } else {
            Err("line needs `extern crate NAME;`")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_map_with_lvls_test() {
        let map = vec![
            ("one".into(), SourceCode::new()),
            ("one/two".into(), SourceCode::new()),
            ("one/two/three".into(), SourceCode::new()),
            ("lib".into(), SourceCode::new()),
            ("two".into(), SourceCode::new()),
        ]
        .into_iter()
        .collect();
        let mut i = mods_map_with_lvls(&map).map(|x| (x.0, x.1));

        assert_eq!(i.next(), Some((0, 1)));
        assert_eq!(i.next(), Some((1, 2)));
        assert_eq!(i.next(), Some((2, 3)));
        assert_eq!(i.next(), Some((3, 1)));
        assert_eq!(i.next(), None);
    }

    #[test]
    fn test_parse_crate() {
        let err = Err("line needs `extern crate NAME;`");
        let c = CrateType::parse_str("extern crat name;");
        assert_eq!(c, err);

        let c = CrateType::parse_str("extern crate  ");
        assert_eq!(c, err);

        let c = CrateType::parse_str("extern crate ;");
        assert_eq!(c, err);

        let s = String::from("extern crate somelib;");
        let c = CrateType::parse_str(&s);
        assert_eq!(
            c,
            Ok(CrateType {
                src_line: s,
                cargo_name: String::from("somelib"),
            })
        );

        let s = String::from("extern crate some-lib;");
        let c = CrateType::parse_str(&s);
        assert_eq!(
            c,
            Ok(CrateType {
                src_line: s,
                cargo_name: String::from("some-lib"),
            })
        );

        let s = String::from("extern crate some lib;");
        let c = CrateType::parse_str(&s);
        assert_eq!(
            c,
            Ok(CrateType {
                src_line: s,
                cargo_name: String::from("some"),
            })
        );

        let s = String::from("extern crate some_lib;");
        let c = CrateType::parse_str(&s);
        assert_eq!(
            c,
            Ok(CrateType {
                src_line: s,
                cargo_name: String::from("some-lib"),
            })
        );
    }

    #[test]
    fn assign_let_binding_test() {
        let mut grp = StmtGrp(vec![]);

        let mut s = String::new();
        grp.assign_let_binding(0, &mut s);

        let ans = "";
        assert_eq!(&s, ans);
        assert_eq!(grp.assign_let_binding_length(0), ans.len());

        grp.0.push(Statement {
            expr: "a".to_string(),
            semi: false,
        });

        let mut s = String::new();
        grp.assign_let_binding(0, &mut s);

        let ans = "let out0 = a;";
        assert_eq!(&s, ans);
        assert_eq!(grp.assign_let_binding_length(0), ans.len());

        grp.0.push(Statement {
            expr: "b".to_string(),
            semi: false,
        });

        let mut s = String::new();
        grp.assign_let_binding(0, &mut s);

        let ans = "a\nlet out0 = b;";
        assert_eq!(&s, ans);
        assert_eq!(grp.assign_let_binding_length(0), ans.len());

        let mut s = String::new();
        grp.assign_let_binding(100, &mut s);

        let ans = "a\nlet out100 = b;";
        assert_eq!(&s, ans);
        assert_eq!(grp.assign_let_binding_length(100), ans.len());
    }

    #[test]
    fn construct_test() {
        use linking::LinkingConfiguration;

        let mut src_code = SourceCode::new();
        let mod_path: &[&str] = &[];
        let linking_config = LinkingConfiguration::default();

        let mut s = String::new();
        append_buffer(&src_code, &mod_path, &linking_config, &mut s);
        let (len, rng) = append_buffer_length(&src_code, &mod_path, &linking_config);

        let ans = r##"#[no_mangle]
pub extern "C" fn _intern_eval() -> kserd::Kserd<'static> {
kserd::Kserd::new_str("no statements")
}
"##;
        assert_eq!(&s, ans);
        assert_eq!(len, ans.len());
        assert_eq!(rng, 73..111);
        assert_eq!(&ans[rng], r#"kserd::Kserd::new_str("no statements")"#);

        // alter mod path
        let mod_path = ["some".to_string(), "path".to_string()];

        let mut s = String::new();
        append_buffer(&src_code, &mod_path, &linking_config, &mut s);
        let (len, rng) = append_buffer_length(&src_code, &mod_path, &linking_config);

        let ans = r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval() -> kserd::Kserd<'static> {
kserd::Kserd::new_str("no statements")
}
"##;
        assert_eq!(&s, ans);
        assert_eq!(len, ans.len());
        assert_eq!(rng, 83..121);
        assert_eq!(&ans[rng], r#"kserd::Kserd::new_str("no statements")"#);

        // alter the linking config
        let linking_config = LinkingConfiguration {
            data_type: Some("String".to_string()),
            ..Default::default()
        };

        let mut s = String::new();
        append_buffer(&src_code, &mod_path, &linking_config, &mut s);
        let (len, rng) = append_buffer_length(&src_code, &mod_path, &linking_config);

        let ans = r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval(app_data: &String) -> kserd::Kserd<'static> {
kserd::Kserd::new_str("no statements")
}
"##;
        assert_eq!(&s, ans);
        assert_eq!(len, ans.len());
        assert_eq!(rng, 100..138);
        assert_eq!(&ans[rng], r#"kserd::Kserd::new_str("no statements")"#);

        // add an item and new input
        src_code.items.push("fn a() {}".to_string());
        src_code.items.push("fn b() {}".to_string());

        let mut s = String::new();
        append_buffer(&src_code, &mod_path, &linking_config, &mut s);
        let (len, rng) = append_buffer_length(&src_code, &mod_path, &linking_config);

        let ans = r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval(app_data: &String) -> kserd::Kserd<'static> {
kserd::Kserd::new_str("no statements")
}
fn a() {}
fn b() {}
"##;
        assert_eq!(&s, ans);
        assert_eq!(len, ans.len());
        assert_eq!(rng, 100..138);
        assert_eq!(&ans[rng], r#"kserd::Kserd::new_str("no statements")"#);

        // add stmts
        src_code.stmts.push(StmtGrp(vec![
            Statement {
                expr: "let a = 1".to_string(),
                semi: true,
            },
            Statement {
                expr: "b".to_string(),
                semi: false,
            },
        ]));
        src_code.stmts.push(StmtGrp(vec![
            Statement {
                expr: "let c = 2".to_string(),
                semi: true,
            },
            Statement {
                expr: "d".to_string(),
                semi: false,
            },
        ]));

        let mut s = String::new();
        append_buffer(&src_code, &mod_path, &linking_config, &mut s);
        let (len, rng) = append_buffer_length(&src_code, &mod_path, &linking_config);

        let ans = r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval(app_data: &String) -> kserd::Kserd<'static> {
let a = 1;
let out0 = b;
let c = 2;
let out1 = d;
kserd::ToKserd::into_kserd(out1).unwrap().to_owned()
}
fn a() {}
fn b() {}
"##;
        assert_eq!(&s, ans);
        assert_eq!(len, ans.len());
        assert_eq!(rng, 150..202);
        assert_eq!(
            &ans[rng],
            "kserd::ToKserd::into_kserd(out1).unwrap().to_owned()"
        );
    }

    #[test]
    fn construct_src_test() {
        // purely tests module adding
        let v = SourceCode::new();

        let linking = LinkingConfiguration::default();
        let map = vec![
            ("lib".into(), v.clone()),
            ("test".into(), v.clone()),
            ("foo/bar".into(), v.clone()),
            ("test/inner".into(), v.clone()),
            ("foo".into(), v.clone()),
            ("test/inner2".into(), v.clone()),
        ]
        .into_iter()
        .collect();

        let (s, map) = construct_source_code(&map, &linking);

        let ans = r##"#[no_mangle]
pub extern "C" fn _lib_intern_eval() -> kserd::Kserd<'static> {
kserd::Kserd::new_str("no statements")
}
mod foo {
#[no_mangle]
pub extern "C" fn _foo_intern_eval() -> kserd::Kserd<'static> {
kserd::Kserd::new_str("no statements")
}
mod bar {
#[no_mangle]
pub extern "C" fn _foo_bar_intern_eval() -> kserd::Kserd<'static> {
kserd::Kserd::new_str("no statements")
}
}}
mod test {
#[no_mangle]
pub extern "C" fn _test_intern_eval() -> kserd::Kserd<'static> {
kserd::Kserd::new_str("no statements")
}
mod inner {
#[no_mangle]
pub extern "C" fn _test_inner_intern_eval() -> kserd::Kserd<'static> {
kserd::Kserd::new_str("no statements")
}
}
mod inner2 {
#[no_mangle]
pub extern "C" fn _test_inner2_intern_eval() -> kserd::Kserd<'static> {
kserd::Kserd::new_str("no statements")
}
}}"##;

        let return_stmt = r#"kserd::Kserd::new_str("no statements")"#;
        assert_eq!(&s, ans);
        assert_eq!(
            &ans[map.get(Path::new("lib")).unwrap().clone()],
            return_stmt
        );
        assert_eq!(
            &ans[map.get(Path::new("foo")).unwrap().clone()],
            return_stmt
        );
        assert_eq!(
            &ans[map.get(Path::new("foo/bar")).unwrap().clone()],
            return_stmt
        );
        assert_eq!(
            &ans[map.get(Path::new("test")).unwrap().clone()],
            return_stmt
        );
        assert_eq!(
            &ans[map.get(Path::new("test/inner")).unwrap().clone()],
            return_stmt
        );
        assert_eq!(
            &ans[map.get(Path::new("test/inner2")).unwrap().clone()],
            return_stmt
        );
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
}
