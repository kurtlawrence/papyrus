//! Pertains to every required for a source file contents.

use linking;

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
    linking_config: Option<&linking::LinkingConfiguration>,
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
        ::pfh::eval_fn_name(mod_path),
        ::pfh::fn_args(linking_config),
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
        let line = match string.trim().split("\n").nth(0) {
            Some(s) => Ok(s),
            None => Err("string should have one line"),
        }?;
        if line.contains("extern crate ") {
            match line
                .split(" ")
                .nth(2)
                .map(|s| s.replace(";", "").replace("_", "-"))
            {
                Some(s) => Ok(CrateType {
                    src_line: line.to_string(),
                    cargo_name: s,
                }),
                None => Err("no crate name"),
            }
        } else {
            Err("line needs `extern crate `")
        }
    }
}
