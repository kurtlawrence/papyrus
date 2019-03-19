use super::*;

use crate::pfh::linking::{Brw, BrwMut, NoRef};
use linefeed::terminal::{DefaultTerminal, Terminal};
use std::io;

impl<Data, Ref> Default for Repl<Read, DefaultTerminal, Data, Ref> {
	fn default() -> Self {
		let mut data = ReplData::default();
		data.redirect_on_execution = false;
		let terminal1 =
			linefeed::terminal::DefaultTerminal::new().expect("failed to start default terminal");
		let terminal2 =
			linefeed::terminal::DefaultTerminal::new().expect("failed to start default terminal");
		Repl {
			state: Read,
			terminal: ReplTerminal {
				terminal: Arc::new(terminal1),
				input_rdr: InputReader::with_term("papyrus", terminal2)
					.expect("failed to start input reader"),
			},
			data: data,
			data_mrker: PhantomData,
			ref_mrker: PhantomData,
		}
	}
}

impl<Term: Terminal + Clone, Data, Ref> Repl<Read, Term, Data, Ref> {
	pub fn with_term(terminal: Term) -> Self {
		let data = ReplData::default();
		let terminal2 = terminal.clone();
		Repl {
			state: Read,
			terminal: ReplTerminal {
				terminal: Arc::new(terminal),
				input_rdr: InputReader::with_term("papyrus", terminal2)
					.expect("failed to start input reader"),
			},
			data: data,
			data_mrker: PhantomData,
			ref_mrker: PhantomData,
		}
	}
}

impl<Term: Terminal, Data, Ref> Repl<Read, Term, Data, Ref> {
	/// Reads input from the input reader until an evaluation phase can begin.
	pub fn read(mut self) -> Repl<Evaluate, Term, Data, Ref> {
		let mut more = false;
		let treat_as_cmd = !self.data.cmdtree.at_root();
		loop {
			let prompt = self.prompt(more);

			let result = self.terminal.input_rdr.read_input(&prompt, treat_as_cmd);

			more = match &result {
				InputResult::Empty => more,
				InputResult::More => true,
				_ => false,
			};

			if !more {
				return self.move_state(Evaluate { result });
			}
		}
	}

	pub fn push_input(mut self, input: char) -> PushResult<Term, Data, Ref> {
		let prompt = self.prompt(false);
		let treat_as_cmd = !self.data.cmdtree.at_root();
		match self
			.terminal
			.input_rdr
			.push_input(&prompt, treat_as_cmd, input)
		{
			Some(result) => {
				if result == InputResult::More {
					PushResult::Read(self)
				} else {
					PushResult::Eval(self.move_state(Evaluate { result }))
				}
			}
			None => PushResult::Read(self),
		}
	}

	fn prompt(&self, more: bool) -> String {
		let s = self.data.cmdtree.path().color(self.data.prompt_colour);
		if more {
			format!("{}.> ", s)
		} else {
			format!("{}=> ", s)
		}
	}
}

impl<Term: Terminal, Data: Copy> Repl<Read, Term, Data, NoRef> {
	/// Run the REPL interactively. Consumes the REPL in the process and will block this thread until exited.
	/// Data must implement `Copy` such that it can loop.
	///
	/// # Panics
	/// - Failure to initialise `InputReader`.
	pub fn run(self, app_data: Data) {
		output_ver(self.terminal.terminal.as_ref());

		let mut read = self;
		loop {
			match read.read().eval(app_data) {
				Ok(r) => read = r.print(),
				Err(sig) => match sig {
					EvalSignal::Exit => break,
				},
			}
		}
	}
}

impl<Term: Terminal, Data> Repl<Read, Term, Data, Brw> {
	/// Run the REPL interactively. Consumes the REPL in the process and will block this thread until exited.
	///
	/// # Panics
	/// - Failure to initialise `InputReader`.
	pub fn run(self, app_data: &Data) {
		output_ver(self.terminal.terminal.as_ref());

		let mut read = self;
		loop {
			match read.read().eval(app_data) {
				Ok(r) => read = r.print(),
				Err(sig) => match sig {
					EvalSignal::Exit => break,
				},
			}
		}
	}
}

impl<Term: Terminal, Data> Repl<Read, Term, Data, BrwMut> {
	/// Run the REPL interactively. Consumes the REPL in the process and will block this thread until exited.
	///
	/// # Panics
	/// - Failure to initialise `InputReader`.
	pub fn run(self, app_data: &mut Data) {
		output_ver(self.terminal.terminal.as_ref());

		let mut read = self;
		loop {
			match read.read().eval(app_data) {
				Ok(r) => read = r.print(),
				Err(sig) => match sig {
					EvalSignal::Exit => break,
				},
			}
		}
	}
}

fn output_ver<T: Terminal>(term: &T) {
	cratesiover::output_to_writer("papyrus", env!("CARGO_PKG_VERSION"), &mut Writer(term)).unwrap();
}
