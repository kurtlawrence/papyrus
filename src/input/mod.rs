use super::*;
use linefeed::terminal::{DefaultTerminal, Terminal};
use linefeed::{Interface, ReadResult};
use syn::Expr;

mod parse;
#[cfg(test)]
mod tests;

pub use self::parse::parse_command;
pub use self::parse::parse_program;

/// Reads input from `stdin`.
pub struct InputReader<Term: Terminal> {
    buffer: String,
    pub interface: Interface<Term>,
}

/// Possible results from reading input from `InputReader`
#[derive(Debug, PartialEq)]
pub enum InputResult {
    /// Command argument as `(command_name, rest_of_line)`.
    Command(String, String),
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

/// Represents an input program.
#[derive(Debug, PartialEq)]
pub struct Input {
    /// Module-level items (`fn`, `enum`, `type`, `struct`, etc.)
    pub items: Vec<String>,
    /// Inner statements and declarations.
    pub stmts: Vec<Statement>,
    /// The referenced crates.
    pub crates: Vec<CrateType>,
}

/// Represents an inner statement.
#[derive(Debug, PartialEq)]
pub struct Statement {
    /// The code, not including the trailing semi if there is one.
    pub expr: String,
    /// Flags whether there is a trailing semi.
    pub semi: bool,
}

impl InputReader<DefaultTerminal> {
    /// Constructs a new `InputReader` reading from `stdin`.
    pub fn new(app_name: &'static str) -> Result<Self, String> {
        let r = match Interface::new(app_name) {
            Ok(r) => r,
            Err(e) => return Err(format!("failed to initialise interface: {}", e)),
        };
        Ok(InputReader {
            buffer: String::new(),
            interface: r,
        })
    }
}

impl<Term: Terminal> InputReader<Term> {
    /// Creates an `InputReader` with the specified terminal.
    pub fn with_term(app_name: &'static str, term: Term) -> Result<Self, String> {
        let r = match Interface::with_term(app_name, term) {
            Ok(r) => r,
            Err(e) => return Err(format!("failed to initialise interface: {}", e)),
        };
        Ok(InputReader {
            buffer: String::new(),
            interface: r,
        })
    }

    /// Reads a single command, item, or statement from `stdin`.
    /// Returns `More` if further input is required for a complete result.
    /// In this case, the input received so far is buffered internally.
    pub fn read_input(&mut self, prompt: &str) -> InputResult {
        self.interface
            .set_prompt(prompt)
            .expect("failed to set prompt");
        let mut r = self
            .interface
            .read_line_step(Some(std::time::Duration::from_millis(10)));
        while let Ok(None) = r {
            std::thread::sleep(std::time::Duration::from_millis(10));
            r = self
                .interface
                .read_line_step(Some(std::time::Duration::from_millis(10)));
        }
        let r = r
            .unwrap_or(Some(ReadResult::Eof))
            .expect("should always be some by this point");
        self.handle_input(r)
    }

    fn handle_input(&mut self, result: ReadResult) -> InputResult {
        let line = match result {
            ReadResult::Eof => return InputResult::Eof,
            ReadResult::Input(s) => s,
            ReadResult::Signal(_) => {
                self.buffer.clear();
                return InputResult::Empty;
            }
        };
        let r = self.determine_result(&line);
        match &r {
            InputResult::Empty => (),
            _ => self.interface.add_history(line),
        }

        r
    }

    fn determine_result(&mut self, line: &str) -> InputResult {
        debug!("input value: {}", line);

        self.buffer.push_str(&line);
        if self.buffer.is_empty() {
            return InputResult::Empty;
        }

        let res = if is_command(&line) {
            parse_command(&line)
        } else {
            // check if the final statement ends with a semi
            match parse_program(&self.buffer) {
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

        match res {
            InputResult::More => (),
            _ => self.buffer.clear(),
        };

        res
    }
}

fn is_command(line: &str) -> bool {
    line.starts_with(".") && !line.starts_with("..")
}
