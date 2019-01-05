use super::command::Commands;
use super::*;

use linefeed::terminal::{DefaultTerminal, Terminal};
use std::io::Read as IoRead;
use std::io::{self, BufRead};

impl<'data, Term: Terminal> Repl<'data, ManualPrint, Term> {
	/// asdfdsa
	pub fn print(self, result_output: &str) -> Repl<'data, Print, Term> {
		let to_print = result_output.to_string();
		Repl {
			state: Print {
				to_print: to_print,
				as_out: false,
			},
			terminal: self.terminal,
			data: self.data,
		}
	}
}

impl<'d, Term: Terminal> Repl<'d, Print, Term> {
	/// Prints the result if successful as `[out#]` or the failure message if any.
	pub fn print(self) -> Repl<'d, Read, Term> {
		let Repl {
			state,
			terminal,
			data,
		} = self;

		// write
		{
			if state.as_out {
				let out_stmt = format!("[out{}]", data.statements.len().saturating_sub(1));
				writeln!(
					Writer(&terminal.terminal),
					"{} {}: {}",
					data.name.color(data.prompt_colour),
					out_stmt.color(data.out_colour),
					state.to_print
				)
				.expect("failed writing");
			} else {
				writeln!(Writer(&terminal.terminal), "{}", state.to_print).expect("failed writing");
			}
		}

		Repl {
			state: Read,
			terminal: terminal,
			data: data,
		}
	}
}