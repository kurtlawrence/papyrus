use super::*;

use linefeed::terminal::{DefaultTerminal, Terminal};
use std::io;

impl<'data, Data> Repl<'data, Read, DefaultTerminal, Data> {
    pub fn default_terminal(data: &'data mut ReplData<Data>) -> Self {
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

impl<'data, Term: Terminal + Clone, Data> Repl<'data, Read, Term, Data> {
    pub fn with_term(terminal: Term, data: &'data mut ReplData<Data>) -> Self {
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

impl<'data, Term: Terminal, Data> Repl<'data, Read, Term, Data> {
    /// Reads input from the input reader until an evaluation phase can begin.
    pub fn read(mut self) -> Repl<'data, Evaluate, Term, Data> {
        // if the cmdr is not sitting at root then always just send it as a command
        let treat_as_cmd = !self.data.cmdtree.at_root();
        let mut more = false;
        loop {
            let prompt = if more {
                format!(
                    "{}.> ",
                    self.data.cmdtree.path().color(self.data.prompt_colour)
                )
            } else {
                format!(
                    "{}=> ",
                    self.data.cmdtree.path().color(self.data.prompt_colour)
                )
            };

            let result = self.terminal.input_rdr.read_input(&prompt, treat_as_cmd);

            more = match &result {
                InputResult::Command(_) => false,
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
}

impl<'data, Term: Terminal, Data: Copy> Repl<'data, Read, Term, Data> {
    /// Run the REPL interactively. Consumes the REPL in the process and will block this thread until exited.
    ///
    /// # Panics
    /// - Failure to initialise `InputReader`.
    pub fn run(self, app_data: Data) {
        query_and_print_ver_info(&self.terminal.terminal);
        let mut read = self;

        loop {
            let eval = read.read();
            let print = eval.eval(app_data);
            match print {
                Ok(r) => read = r.print(),
                Err(sig) => match sig {
					EvalSignal::Exit => break,
				}
            }
        }
    }
}

fn query_and_print_ver_info<Term: Terminal>(terminal: &Term) {
    use cratesiover;
    cratesiover::output_with_term("papyrus", env!("CARGO_PKG_VERSION"), terminal);
}
