use super::*;

use linefeed::terminal::Terminal;

impl<'data, Term: Terminal, Arg, Data> Repl<'data, ManualPrint, Term, Arg, Data> {
	/// asdfdsa
	pub fn print(self, result_output: &str) -> Repl<'data, Print, Term, Arg, Data> {
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

impl<'d, Term: Terminal, Arg, Data> Repl<'d, Print, Term, Arg, Data> {
	/// Prints the result if successful as `[out#]` or the failure message if any.
	pub fn print(self) -> Repl<'d, Read, Term, Arg, Data> {
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
					Writer(&terminal.terminal),
					"{} {}: {}",
					data.name.color(data.prompt_colour),
					out_stmt.color(data.out_colour),
					state.to_print
				)
				.expect("failed writing");
			} else {
				if state.to_print.len() > 0 {
					// only write if there is something to write.
					writeln!(Writer(&terminal.terminal), "{}", state.to_print)
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
