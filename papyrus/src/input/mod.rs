use crate::code::{CrateType, Input};
use syn::Expr;

mod parse;
#[cfg(test)]
mod tests;

pub use self::parse::parse_command;
pub use self::parse::parse_program;

/// Possible results from reading input from `InputReader`
#[derive(Debug, PartialEq)]
pub enum InputResult {
    /// Command or commands in a line.
    Command(String),
    /// Code as input
    Program(Input),
    /// An empty line
    Empty,
    /// Needs more input; i.e. there is an unclosed delimiter.
    More,
    /// End of file reached.
    Eof,
    /// Error while parsing input.
    InputError(String),
}

/// Parse `input` and `line` and determine what `InputResult`.
pub fn determine_result(input: &str, line: &str, treat_as_cmd: bool) -> InputResult {
    if input.is_empty() {
        return InputResult::Empty; // if line is empty this could result. do not remove
    }

    let res = if treat_as_cmd || is_command(line) {
        parse_command(line)
    } else {
        // check if the final statement ends with a semi
        match parse_program(input) {
            InputResult::Program(input) => {
                if input.stmts.len() > 0 && input.stmts.last().unwrap().semi {
                    InputResult::More
                } else {
                    InputResult::Program(input)
                }
            }
            x => x,
        }
    };

    res
}

fn is_command(line: &str) -> bool {
    line.starts_with(crate::CMD_PREFIX)
}
