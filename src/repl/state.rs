use super::command::Commands;
use super::*;
use linefeed::terminal::{DefaultTerminal, Terminal};
use std::io::Read as IoRead;
use std::io::{self, BufRead};

impl<'data, S, Term: Terminal> Repl<'data, S, Term> {
	/// Load a file into the repl, no matter the current state. Returns a repl awaiting evaluation.
	pub fn load<P: AsRef<Path>>(self, file_path: P) -> Repl<'data, Evaluate, Term> {
		let result = load_and_parse(file_path);
		Repl {
			state: Evaluate { result },
			terminal: self.terminal,
			data: self.data,
		}
	}

	// TODO make this clean the repl as well.
	pub fn clean(&self) {
		match compile_dir().canonicalize() {
			Ok(d) => {
				let target_dir = format!("{}/target", d.to_string_lossy());
				fs::remove_dir_all(target_dir).is_ok();
			}
			_ => (),
		}
	}
}

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
				Err(_) => "Failed to query crates.io".to_string(),
			};
			let mut wtr = Writer(self.terminal.terminal.lock_write());
			wtr.overwrite_current_console_line(&print_line).unwrap();
			writeln!(wtr, "",).unwrap();
		} // version information.

		let mut read = self;

		loop {
			let eval = read.read();
			let print = eval.eval();
			match print {
				Ok(r) => read = r.print(),
				Err(_) => break,
			}
		}
	}
}

impl<'data, Term: Terminal> Repl<'data, Evaluate, Term> {
	/// Evaluates the read input, compiling and executing the code and printing all line prints until a result is found.
	/// This result gets passed back as a print ready repl.
	pub fn eval(self) -> Result<Repl<'data, Print, Term>, ()> {
		let Repl {
			state,
			terminal,
			mut data,
		} = self;

		let (to_print, as_out) = match state.result {
			InputResult::Command(name, args) => {
				debug!("read command: {} {:?}", name, args);
				match data.commands.find_command(&name) {
					Err(e) => (e.to_string(), false),
					Ok(cmd) => {
						return (cmd.action)(
							Repl {
								state: ManualPrint,
								terminal: terminal,
								data: data,
							},
							&args,
						);
					}
				}
			}
			InputResult::Program(input) => {
				debug!("read program: {:?}", input);
				match handle_input(&mut data, input, &terminal.terminal) {
					Ok((s, as_out)) => (s, as_out),
					Err(s) => (s, false),
				}
			}
			InputResult::Eof => return Err(()),
			InputResult::InputError(err) => (err, false),
			_ => (String::new(), false),
		};
		Ok(Repl {
			state: Print { to_print, as_out },
			terminal: terminal,
			data: data,
		})
	}
}

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
			let mut wtr = Writer(terminal.terminal.lock_write());
			if state.as_out {
				let out_stmt = format!("[out{}]", data.statements.len().saturating_sub(1));
				writeln!(
					wtr,
					"{} {}: {}",
					data.name.color(data.prompt_colour),
					out_stmt.color(data.out_colour),
					state.to_print
				)
				.expect("failed writing");
			} else {
				writeln!(wtr, "{}", state.to_print).expect("failed writing");
			}
		}

		Repl {
			state: Read,
			terminal: terminal,
			data: data,
		}
	}
}

type HandleInputResult = (String, bool);

/// Runs a single program input.
fn handle_input<Term: Terminal>(
	data: &mut ReplData<Term>,
	input: Input,
	terminal: &Term,
) -> Result<HandleInputResult, String> {
	let additionals = build_additionals(input, data.statements.len());
	let src = build_source(data, additionals.clone());
	match eval(&compile_dir(), src, terminal) {
		Ok(s) => {
			//Successful compile/runtime means we can add the new items to every program
			// crates
			additionals
				.crates
				.into_iter()
				.for_each(|c| data.crates.push(c));

			// items
			if let Some(items) = additionals.items {
				data.items.push(items);
			}

			// statements
			let mut as_out = false;
			if let Some(stmts) = additionals.stmts {
				data.statements.push(stmts.stmts);
				as_out = true;
			}
			Ok((s, as_out))
		}
		Err(s) => Err(s),
	}
}

fn build_additionals(input: Input, statement_num: usize) -> Additional {
	let mut additional_items = None;
	let mut additional_statements = None;
	let mut print_stmt = String::new();
	let Input {
		items,
		mut stmts,
		crates,
	} = input;

	if items.len() > 0 {
		additional_items = Some(items);
	}
	if stmts.len() > 0 {
		if let Some(mut last) = stmts.pop() {
			let expr = if !last.semi {
				print_stmt = format!(
					"println!(\"{}{{:?}}\", out{});",
					PAPYRUS_SPLIT_PATTERN, statement_num
				);
				format!("let out{} = {};", statement_num, last.expr)
			} else {
				last.expr.to_string()
			};
			last.expr = expr;
			stmts.push(last);
		}
		let stmts = stmts
			.into_iter()
			.map(|mut x| {
				if x.semi {
					x.expr.push(';');
				}
				x.expr
			})
			.collect();
		additional_statements = Some(AdditionalStatements { stmts, print_stmt });
	}

	Additional {
		items: additional_items,
		stmts: additional_statements,
		crates: crates,
	}
}

fn build_source<Term: Terminal>(data: &mut ReplData<Term>, additional: Additional) -> SourceFile {
	let mut items = data
		.items
		.iter()
		.flatten()
		.map(|x| x.to_owned())
		.collect::<Vec<_>>()
		.join("\n");
	let mut statements = data
		.statements
		.iter()
		.flatten()
		.map(|x| x.to_owned())
		.collect::<Vec<_>>()
		.join("\n");
	let crates = data
		.crates
		.iter()
		.chain(additional.crates.iter())
		.map(|x| x.clone())
		.collect();
	if let Some(i) = additional.items {
		items.push_str("\n");
		items.push_str(&i.join("\n"));
	}
	if let Some(stmts) = additional.stmts {
		statements.push('\n');
		statements.push_str(&stmts.stmts.join("\n"));
		statements.push('\n');
		statements.push_str(&stmts.print_stmt);
	}

	SourceFile {
		src: code(&statements, &items),
		file_name: String::from("mem-code"),
		file_type: SourceFileType::Rs,
		crates: crates,
	}
}

/// Evaluates the source file by compiling and running the given source file.
/// Returns the stderr if unsuccessful compilation or runtime, or the evaluation print value.
/// Stderr is piped to the current stdout for compilation, with each line overwriting itself.
fn eval<P: AsRef<Path>, Term: Terminal>(
	compile_dir: P,
	source: SourceFile,
	terminal: &Term,
) -> Result<String, String> {
	let mut c = Exe::compile(&source, compile_dir).unwrap();
	let mut wtr = Writer(terminal.lock_write());

	let compilation_stderr = {
		// output stderr stream line by line, erasing each line as you go.
		let rdr = BufReader::new(c.stderr());
		let mut s = String::new();
		for line in rdr.lines() {
			let line = line.unwrap();
			wtr.overwrite_current_console_line(&line).unwrap();
			s.push_str(&line);
			s.push('\n');
		}
		wtr.overwrite_current_console_line("").unwrap();
		s
	};

	match c.wait() {
		Ok(exe) => {
			let mut c = exe.run(&::std::env::current_dir().unwrap());
			// print out the stdout as each line comes
			// split out on the split pattern, and do not print that section!
			let print = {
				let rdr = BufReader::new(c.stdout());
				let mut s = String::new();
				for line in rdr.lines() {
					let line = line.unwrap();
					let mut split = line.split(PAPYRUS_SPLIT_PATTERN);
					if let Some(first) = split.next() {
						if !first.is_empty() {
							writeln!(wtr, "{}", first).unwrap();
						}
					}
					if let Some(second) = split.next() {
						s.push_str(second);
					}
				}
				s
			};

			let stderr = {
				let mut s = String::new();
				let mut rdr = BufReader::new(c.stderr());
				rdr.read_to_string(&mut s).unwrap();
				s
			};

			if c.wait().success() {
				Ok(print)
			} else {
				Err(stderr)
			}
		}
		Err(_) => Err(compilation_stderr),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn load_rs_source() {
		let mut data = ReplData::default();
		for src_file in RS_FILES.iter() {
			let file = format!("test-src/{}", src_file);
			println!("{}", file);
			let res = load_and_parse(&file);
			match res {
				InputResult::Program(input) => {
					let additionals = build_additionals(input, data.statements.len());
					let src = build_source(&mut data, additionals);
					let eval = eval(
						&format!("test/{}", src_file.split(".").nth(0).unwrap()),
						src,
					);
					let b = eval.is_ok();
					if let Err(e) = eval {
						println!("{}", e);
					}
					assert!(b);
				}
				InputResult::InputError(e) => {
					println!("{}", e);
					panic!("should have parsed as program, got input error")
				}
				InputResult::More => panic!("should have parsed as program, got more"),
				InputResult::Command(_, _) => panic!("should have parsed as program, got command"),
				InputResult::Empty => panic!("should have parsed as program, got empty"),
				InputResult::Eof => panic!("should have parsed as program, got Eof"),
			}
		}
	}

	#[test]
	fn load_rscript_script() {
		let mut data = ReplData::default();
		for src_file in RSCRIPT_FILES.iter() {
			let file = format!("test-src/{}", src_file);
			println!("{}", file);
			let res = load_and_parse(&file);
			match res {
				InputResult::Program(input) => {
					let additionals = build_additionals(input, data.statements.len());
					let src = build_source(&mut data, additionals);
					let eval = eval(
						&format!("test/{}", src_file.split(".").nth(0).unwrap()),
						src,
					);
					let b = eval.is_ok();
					if let Err(e) = eval {
						println!("{}", e);
					}
					assert!(b);
				}
				InputResult::InputError(e) => {
					println!("{}", e);
					panic!("should have parsed as program, got input error")
				}
				InputResult::More => panic!("should have parsed as program, got more"),
				InputResult::Command(_, _) => panic!("should have parsed as program, got command"),
				InputResult::Empty => panic!("should have parsed as program, got empty"),
				InputResult::Eof => panic!("should have parsed as program, got Eof"),
			}
		}
	}
}
