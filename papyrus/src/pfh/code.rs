//! Pertains to everything required for a source file contents.
use super::*;
use crate::pfh::linking;
use linking::LinkingConfiguration;
use std::path::Path;

#[derive(Debug, PartialEq, Clone)]
pub struct Input {
    /// Module-level items (`fn`, `enum`, `type`, `struct`, etc.)
    pub items: Vec<Item>,
    /// Inner statements and declarations.
    pub stmts: Vec<Statement>,
    /// The referenced crates.
    pub crates: Vec<CrateType>,
}

impl Input {
    /// Stringfy's the statements and assigns trailing expressions with `let out# = expr;`.
    fn assign_let_binding(&self, input_num: usize, buf: &mut String) {
        for stmt in &self.stmts[0..self.stmts.len().saturating_sub(1)] {
            buf.push_str(&stmt.expr);
            if stmt.semi {
                buf.push(';');
            }
            buf.push('\n');
        }

        if self.stmts.len() > 0 {
            buf.push_str("let out");
            buf.push_str(&input_num.to_string());
            buf.push_str(" = ");
            buf.push_str(&self.stmts[self.stmts.len() - 1].expr);
            buf.push(';');
        }
    }
}

pub type SourceCode = Vec<Input>;

pub fn construct_source_code(file_map: &FileMap, linking_config: &LinkingConfiguration) -> String {
    // assumed to be sorted, FileMap is BTreeMap

    let cap = calc_cap(file_map, linking_config);

    let mut contents = String::with_capacity(cap);

    // add in external crates
    for external in linking_config.external_libs.iter() {
        external.construct_code_str(&mut contents);
    }

    // do the lib first
    if let Some(lib) = file_map.get(Path::new("lib")) {
        code::append_buffer(
            lib,
            &into_mod_path_vec(Path::new("lib")), // lib is  empty
            linking_config,
            &mut contents,
        );
    }

    let mut lvl = 0;
    for (file, src_code) in file_map.iter() {
        if file == Path::new("lib") {
            continue;
        }

        use std::cmp::Ordering;

        let new_lvl = file.components().count();

        match new_lvl.cmp(&lvl) {
            Ordering::Equal | Ordering::Less => {
                // need to close off the open modules
                let diff = lvl - new_lvl; // should always be >= 0
                for _ in 0..=diff {
                    contents.push('}');
                }
                contents.push('\n');
            }
            _ => (),
        }

        lvl = new_lvl;

        contents.push_str("mod ");
        contents.push_str(
            file.components()
                .last()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap(),
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
    for _ in 0..lvl {
        contents.push('}');
    }

    // debug_assert_eq!(
    //     cap,
    //     contents.len(),
    //     "failed at calculating the correct capacity"
    // );

    contents
}

fn calc_cap(file_map: &FileMap, linking_config: &LinkingConfiguration) -> usize {
    let mut size = 0;

    // add in external crates
    size += linking_config
        .external_libs
        .iter()
        .map(|x| x.calc_code_str_len())
        .sum::<usize>();

    size
}

/// Build the buffer with the stringified contents of SourceCode
///
/// The structure of the file follows:
/// ```txt
/// ##INTERNAL_EVALUATION_FN({stmts})
///
/// {items}
/// ```
///
/// A module _will_ contain **one** evaluation function, qualified with the module path.
/// This evaulation function is what contains the statements.
pub fn append_buffer(
    src_code: &SourceCode,
    mod_path: &[String],
    linking_config: &linking::LinkingConfiguration,
    buf: &mut String,
) {
    // wrap stmts
    buf.push_str("#[no_mangle]\npub extern \"C\" fn "); // 31 len
    eval_fn_name(mod_path, buf);
    buf.push('(');
    linking_config.construct_fn_args(buf);
    buf.push_str(") -> String {\n"); // 14 len

    // add stmts
    let c = src_code.iter().filter(|x| x.stmts.len() > 0).count();
    if c >= 1 {
        // only add statements if more than zero!
        src_code
            .iter()
            .filter(|x| x.stmts.len() > 0)
            .enumerate()
            .for_each(|(i, x)| {
                x.assign_let_binding(i, buf);
                buf.push('\n');
            });
        buf.push_str("format!(\"{:?}\", out");
        buf.push_str(&c.saturating_sub(1).to_string());
        buf.push_str(")\n");
    } else {
        buf.push_str("String::from(\"no statements\")\n");
    }
    buf.push_str("}\n");

    // add items
    for item in src_code.iter().flat_map(|x| &x.items) {
        buf.push_str(&item);
        buf.push('\n');
    }
}

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
    let mut input = Input {
        items: vec![],
        stmts: vec![],
        crates: vec![],
    };

    let mut s = String::new();
    input.assign_let_binding(0, &mut s);
    assert_eq!(&s, "");

    input.items.push("asdf".to_string());
    input
        .crates
        .push(CrateType::parse_str("extern crate rand;").unwrap());

    let mut s = String::new();
    input.assign_let_binding(0, &mut s);
    assert_eq!(&s, ""); // should still be nothing, done on statements

    input.stmts.push(Statement {
        expr: "a".to_string(),
        semi: false,
    });

    let mut s = String::new();
    input.assign_let_binding(0, &mut s);
    assert_eq!(&s, "let out0 = a;");

    input.stmts.push(Statement {
        expr: "b".to_string(),
        semi: false,
    });

    let mut s = String::new();
    input.assign_let_binding(0, &mut s);
    assert_eq!(&s, "a\nlet out0 = b;");

    let mut s = String::new();
    input.assign_let_binding(100, &mut s);
    assert_eq!(&s, "a\nlet out100 = b;");
}

#[test]
fn construct_test() {
    use linking::LinkingConfiguration;

    let mut src_code = vec![Input {
        items: vec![],
        stmts: vec![],
        crates: vec![],
    }];
    let mod_path = [];
    let linking_config = LinkingConfiguration::default();

    let mut s = String::new();
    append_buffer(&src_code, &mod_path, &linking_config, &mut s);
    assert_eq!(
        &s,
        r##"#[no_mangle]
pub extern "C" fn _intern_eval() -> String {
String::from("no statements")
}
"##
    );

    // alter mod path
    let mod_path = ["some".to_string(), "path".to_string()];

    let mut s = String::new();
    append_buffer(&src_code, &mod_path, &linking_config, &mut s);
    assert_eq!(
        &s,
        r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval() -> String {
String::from("no statements")
}
"##
    );

    let mut s = String::new();
    append_buffer(&src_code, &mod_path, &linking_config, &mut s);
    assert_eq!(
        &s,
        r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval() -> String {
String::from("no statements")
}
"##
    );

    // alter the linking config
    let linking_config = LinkingConfiguration {
        data_type: Some("String".to_string()),
        ..Default::default()
    };

    let mut s = String::new();
    append_buffer(&src_code, &mod_path, &linking_config, &mut s);
    assert_eq!(
        &s,
        r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval(app_data: &String) -> String {
String::from("no statements")
}
"##
    );

    // add an item and new input
    src_code[0].items.push("fn a() {}".to_string());
    src_code.push(Input {
        items: vec!["fn b() {}".to_string()],
        stmts: vec![],
        crates: vec![],
    });

    let mut s = String::new();
    append_buffer(&src_code, &mod_path, &linking_config, &mut s);
    assert_eq!(
        &s,
        r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval(app_data: &String) -> String {
String::from("no statements")
}
fn a() {}
fn b() {}
"##
    );

    // add stmts
    src_code[0].stmts.push(Statement {
        expr: "let a = 1".to_string(),
        semi: true,
    });
    src_code[0].stmts.push(Statement {
        expr: "b".to_string(),
        semi: false,
    });
    src_code[1].stmts.push(Statement {
        expr: "let c = 2".to_string(),
        semi: true,
    });
    src_code[1].stmts.push(Statement {
        expr: "d".to_string(),
        semi: false,
    });

    let mut s = String::new();
    append_buffer(&src_code, &mod_path, &linking_config, &mut s);
    assert_eq!(
        &s,
        r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval(app_data: &String) -> String {
let a = 1;
let out0 = b;
let c = 2;
let out1 = d;
format!("{:?}", out1)
}
fn a() {}
fn b() {}
"##
    );
}

#[test]
fn construct_src_test() {
    // purely tests module adding
    let v = vec![Input {
        crates: vec![],
        stmts: vec![],
        items: vec![],
    }];

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

    let s = construct_source_code(&map, &linking);

    assert_eq!(
        &s,
        r##"#[no_mangle]
pub extern "C" fn _lib_intern_eval() -> String {
String::from("no statements")
}
mod foo {
#[no_mangle]
pub extern "C" fn _foo_intern_eval() -> String {
String::from("no statements")
}
mod bar {
#[no_mangle]
pub extern "C" fn _foo_bar_intern_eval() -> String {
String::from("no statements")
}
}}
mod test {
#[no_mangle]
pub extern "C" fn _test_intern_eval() -> String {
String::from("no statements")
}
mod inner {
#[no_mangle]
pub extern "C" fn _test_inner_intern_eval() -> String {
String::from("no statements")
}
}
mod inner2 {
#[no_mangle]
pub extern "C" fn _test_inner2_intern_eval() -> String {
String::from("no statements")
}
}}"##
    );
}