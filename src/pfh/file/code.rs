//! Pertains to every required for a source file contents.

use crate::pfh::linking;

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
    fn assign_let_binding(&self, input_num: usize) -> String {
        let mut s = String::new();
        for stmt in &self.stmts[0..self.stmts.len().saturating_sub(1)] {
            s.push_str(&stmt.expr);
            if stmt.semi {
                s.push(';');
            }
            s.push('\n');
        }

        if self.stmts.len() > 0 {
            s.push_str(&format!(
                "let out{} = {};",
                input_num,
                self.stmts[self.stmts.len() - 1].expr
            ));
        }

        s
    }
}

pub type SourceCode = Vec<Input>;

/// Build the source code as a `String`.
///
/// The structure of the file follows:
/// ```txt
/// {crates}
///
/// ##INTERNAL_EVALUATION_FN({stmts})
///
/// {items}
/// ```
///
/// A module _will_ contain **one** evaluation function, qualified with the module path.
/// This evaulation function is what contains the statements.
pub fn construct(
    src_code: &SourceCode,
    mod_path: &[String],
    linking_config: &linking::LinkingConfiguration,
) -> String {
    let mut code = String::new();

    // add crates
    for c in src_code.iter().flat_map(|x| &x.crates) {
        code.push_str(&c.src_line);
        code.push('\n');
    }

    // wrap stmts
    code.push_str("#[no_mangle]\n");
    code.push_str(&format!(
        r#"pub extern "C" fn {}({}) -> String {{"#,
        crate::pfh::eval_fn_name(mod_path),
        linking_config.construct_fn_args()
    ));
    code.push('\n');
    // add stmts
    let idx = src_code
        .iter()
        .filter(|x| x.stmts.len() > 0)
        .count()
        .saturating_sub(1);
    code.push_str(
        &src_code
            .iter()
            .filter(|x| x.stmts.len() > 0)
            .enumerate()
            .map(|(i, x)| x.assign_let_binding(i))
            .collect::<Vec<String>>()
            .join("\n"),
    );
    code.push('\n');
    code.push_str(&format!("format!(\"{{:?}}\", out{})\n", idx));
    code.push_str("}\n");

    // add items
    for item in src_code.iter().flat_map(|x| &x.items) {
        code.push_str(&item);
        code.push('\n');
    }

    code
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

    let s = input.assign_let_binding(0);
    assert_eq!(&s, "");

    input.items.push("asdf".to_string());
    input
        .crates
        .push(CrateType::parse_str("extern crate rand;").unwrap());

    let s = input.assign_let_binding(0);
    assert_eq!(&s, ""); // should still be nothing, done on statements

    input.stmts.push(Statement {
        expr: "a".to_string(),
        semi: false,
    });

    let s = input.assign_let_binding(0);
    assert_eq!(&s, "let out0 = a;");

    input.stmts.push(Statement {
        expr: "b".to_string(),
        semi: false,
    });

    let s = input.assign_let_binding(0);
    assert_eq!(&s, "a\nlet out0 = b;");

    let s = input.assign_let_binding(100);
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

    let s = construct(&src_code, &mod_path, &linking_config);
    assert_eq!(
        &s,
        r##"#[no_mangle]
pub extern "C" fn __intern_eval() -> String {

format!("{:?}", out0)
}
"##
    );

    // alter mod path
    let mod_path = ["some".to_string(), "path".to_string()];

    let s = construct(&src_code, &mod_path, &linking_config);
    assert_eq!(
        &s,
        r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval() -> String {

format!("{:?}", out0)
}
"##
    );

    let s = construct(&src_code, &mod_path, &linking_config);
    assert_eq!(
        &s,
        r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval() -> String {

format!("{:?}", out0)
}
"##
    );

    // alter the linking config
    let linking_config = LinkingConfiguration {
        data_type: Some("String".to_string()),
        ..Default::default()
    };

    let s = construct(&src_code, &mod_path, &linking_config);
    assert_eq!(
        &s,
        r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval(app_data: &String) -> String {

format!("{:?}", out0)
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

    let s = construct(&src_code, &mod_path, &linking_config);
    assert_eq!(
        &s,
        r##"#[no_mangle]
pub extern "C" fn _some_path_intern_eval(app_data: &String) -> String {

format!("{:?}", out0)
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

    let s = construct(&src_code, &mod_path, &linking_config);
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

    // add crate
    src_code[0]
        .crates
        .push(CrateType::parse_str("extern crate some_crate as some;").unwrap());

    let s = construct(&src_code, &mod_path, &linking_config);
    assert_eq!(
        &s,
        r##"extern crate some_crate as some;
#[no_mangle]
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
