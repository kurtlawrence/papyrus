use super::*;

use linefeed::terminal::Terminal;

impl<Term: Terminal, Data> Repl<ManualPrint, Term, Data> {
	/// asdfdsa
	pub fn print(self, result_output: &str) -> Repl<Print, Term, Data> {
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

impl<Term: Terminal, Data> Repl<Print, Term, Data> {
	/// Prints the result if successful as `[out#]` or the failure message if any.
	pub fn print(self) -> Repl<Read, Term, Data> {
		let Repl {
			state,
			terminal,
			data,
		} = self;

		// write
		{
			if state.as_out {
				let num = data
					.file_map
					.get(&data.current_file)
					.expect("file map does not contain key")
					.contents
					.iter()
					.filter(|x| x.stmts.len() > 0)
					.count()
					.saturating_sub(1);
				let out_stmt = format!("[out{}]", num);
				writeln!(
					Writer(terminal.terminal.as_ref()),
					"{} {}: {}",
					data.name.color(data.prompt_colour),
					out_stmt.color(data.out_colour),
					state.to_print
				)
				.expect("failed writing");
			} else {
				if state.to_print.len() > 0 {
					// only write if there is something to write.
					writeln!(Writer(terminal.terminal.as_ref()), "{}", state.to_print)
						.expect("failed writing");
				}
			}
		}

		Repl {
			state: Read,
			terminal: terminal,
			data: data,
		}
	}
}
