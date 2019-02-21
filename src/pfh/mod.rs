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

pub fn fn_args(config: Option<&linking::LinkingConfiguration>) -> String {
    match config {
        Some(c) => match c.data_type {
            Some(ref d) => match d.arg {
                linking::ArgumentType::Borrow => format!("app_data: &{}::{}", c.crate_name, d.name),
                linking::ArgumentType::BorrowMut => {
                    format!("app_data: &mut {}::{}", c.crate_name, d.name)
                }
            },
            None => "".to_string(),
        },
        None => "".to_string(),
    }
}
