use crate::pfh::{CrateType, Input};
use linefeed::terminal::Terminal;
use linefeed::{Interface, ReadResult};
use syn::Expr;

mod parse;
#[cfg(test)]
mod tests;

pub use self::parse::parse_command;
pub use self::parse::parse_program;
use std::io;

/// Reads input from `stdin`.
pub struct InputReader<Term: Terminal> {
    buffer: String,
    pub interface: Interface<Term>,
}

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

    pub fn set_prompt(&self, prompt: &str) -> io::Result<()> {
        self.interface.set_prompt(prompt)
    }

    /// Reads a single command, item, or statement from `stdin`.
    /// Returns `More` if further input is required for a complete result.
    /// In this case, the input received so far is buffered internally.
    pub fn read_input(&mut self, prompt: &str, treat_as_cmd: bool) -> InputResult {
        self.interface
            .set_prompt(prompt)
            .expect("failed to set prompt");
        let duration = std::time::Duration::from_millis(500);
        let mut r = self.interface.read_line_step(Some(duration));
        while let Ok(None) = r {
            r = self.interface.read_line_step(Some(duration));
        }
        let r = r
            .unwrap_or(Some(ReadResult::Eof))
            .expect("should always be some by this point");
        self.handle_input(r, treat_as_cmd)
    }

    pub fn push_input(
        &mut self,
        prompt: &str,
        treat_as_cmd: bool,
        input_ch: char,
    ) -> Option<InputResult> {
        self.interface
            .set_prompt(prompt)
            .expect("failed to set prompt");
        self.interface.push_input(input_ch.to_string().as_bytes());
        self.interface
            .read_line_step(None)
            .unwrap_or(Some(ReadResult::Eof))
            .map(|r| self.handle_input(r, treat_as_cmd))
    }

    fn handle_input(&mut self, result: ReadResult, treat_as_cmd: bool) -> InputResult {
        let line = match result {
            ReadResult::Eof => return InputResult::Eof,
            ReadResult::Input(s) => s,
            ReadResult::Signal(_) => {
                self.buffer.clear();
                return InputResult::Empty;
            }
        };

        let r = self.determine_result(&line, treat_as_cmd);
        match &r {
            InputResult::Empty => (),
            _ => self.interface.add_history(line),
        }

        r
    }

    fn determine_result(&mut self, line: &str, treat_as_cmd: bool) -> InputResult {
        debug!("input value: {}", line);

        self.buffer.push_str(&line);
        if self.buffer.is_empty() {
            return InputResult::Empty;
        }

        let res = if treat_as_cmd || is_command(&line) {
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