use super::*;

use linefeed::terminal::{DefaultTerminal, Terminal};
use std::io;
use version::*;

impl<'data> Repl<'data, Read, DefaultTerminal> {
    pub fn default_terminal(data: &'data mut ReplData<DefaultTerminal>) -> Self {
        let terminal1 =
            linefeed::terminal::DefaultTerminal::new().expect("failed to start default terminal");
        let terminal2 =
            linefeed::terminal::DefaultTerminal::new().expect("failed to start default terminal");
        Repl {
            state: Read,
            terminal: ReplTerminal {
                terminal: terminal1,
                input_rdr: InputReader::with_term("papyrus", terminal2)
                    .expect("failed to start input reader"),
            },
            data: data,
        }
    }
}

impl<'data, Term: Terminal + Clone> Repl<'data, Read, Term> {
    pub fn with_term(terminal: Term, data: &'data mut ReplData<Term>) -> Self {
        let terminal2 = terminal.clone();
        Repl {
            state: Read,
            terminal: ReplTerminal {
                terminal: terminal,
                input_rdr: InputReader::with_term("papyrus", terminal2)
                    .expect("failed to start input reader"),
            },
            data: data,
        }
    }
}

impl<'data, Term: Terminal> Repl<'data, Read, Term> {
    /// Reads input from the input reader until an evaluation phase can begin.
    pub fn read(mut self) -> Repl<'data, Evaluate, Term> {
        let mut more = false;
        loop {
            let prompt = if more {
                format!("{}.> ", self.data.name.color(self.data.prompt_colour))
            } else {
                format!("{}=> ", self.data.name.color(self.data.prompt_colour))
            };

            let result = self.terminal.input_rdr.read_input(&prompt);

            more = match &result {
                InputResult::Command(_, _) => false,
                InputResult::Program(_) => false,
                InputResult::Empty => more,
                InputResult::More => true,
                InputResult::Eof => false,
                InputResult::InputError(_) => false,
            };

            if !more {
                return Repl {
                    state: Evaluate { result },
                    terminal: self.terminal,
                    data: self.data,
                };
            }
        }
    }

    /// Run the REPL interactively. Consumes the REPL in the process and will block this thread until exited.
    ///
    /// # Panics
    /// - Failure to initialise `InputReader`.
    pub fn run(self) {
        {
            print!("{}", "Checking for later version...".bright_yellow());
            io::stdout().flush().is_ok();
            let print_line = match query() {
                Ok(status) => match status {
                    Status::UpToDate(ver) => format!(
                        "{}{}",
                        "Running the latest papyrus version ".bright_green(),
                        ver.bright_green()
                    ),
                    Status::OutOfDate(ver) => format!(
                        "{}{}{}{}",
                        "The current papyrus version ".bright_red(),
                        env!("CARGO_PKG_VERSION").bright_red(),
                        " is old, please update to ".bright_red(),
                        ver.bright_red()
                    ),
                },
                Err(_) => format!("{}", "Failed to query crates.io".bright_yellow()),
            };
            let mut wtr = Writer(&self.terminal.terminal);
            wtr.overwrite_current_console_line(&print_line).unwrap();
            writeln!(wtr, "",).unwrap();
        } // version information.

        let mut read = self;

        loop {
            let eval = read.read();
            let print = eval.eval(());
            match print {
                Ok(r) => read = r.print(),
                Err(_) => break,
            }
        }
    }
}
