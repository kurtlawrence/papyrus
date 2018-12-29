use super::command::Commands;
use super::*;
use linefeed::terminal::Terminal;
use std::io::Read as IoRead;
use std::io::{self, BufRead};

impl<'data, S> Repl<'data, S> {
	/// Load a file into the repl, no matter the current state. Returns a repl awaiting evaluation.
	pub fn load<P: AsRef<Path>>(self, file_path: P) -> Repl<'data, Evaluate> {
		let result = load_and_parse(file_path);
		Repl {
			state: Evaluate { result },
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

impl<'data> Repl<'data, Read> {
	pub fn new(data: &'data mut ReplData) -> Self {
		Repl {
			state: Read,
			data: data,
		}
	}

	/// Reads input from the input reader until an evaluation phase can begin.
	pub fn read<Term: Terminal>(self, input_rdr: &mut InputReader<Term>) -> Repl<'data, Evaluate> {
		let mut more = false;
		loop {
			let prompt = if more {
				format!("{}.> ", self.data.name.color(self.data.prompt_colour))
			} else {
				format!("{}=> ", self.data.name.color(self.data.prompt_colour))
			};

			let result = input_rdr.read_input(&prompt);

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
			overwrite_current_console_line(&print_line);
			println!("",);
		} // version information.

		let mut input_rdr = InputReader::new(self.data.name).expect("failed to start input reader");
		let mut read = self;

		loop {
			let eval = read.read(&mut input_rdr);
			let print = eval.eval();
			match print {
				Ok(r) => read = r.print(&mut std::io::stdout()), // write to stdout
				Err(_) => break,
			}
		}
	}
}

impl<'data> Repl<'data, Evaluate> {
	/// Evaluates the read input, compiling and executing the code and printing all line prints until a result is found.
	/// This result gets passed back as a print ready repl.
	pub fn eval(self) -> Result<Repl<'data, Print>, ()> {
		let Repl { state, mut data } = self;

		let (to_print, success) = match state.result {
			InputResult::Command(name, args) => {
				debug!("read command: {} {:?}", name, args);
				match data.commands.find_command(&name) {
					Err(e) => (e.to_string(), false),
					Ok(cmd) => {
						return (cmd.action)(
							Repl {
								state: ManualPrint,
								data: data,
							},
							&args,
						);
					}
				}
			}
			InputResult::Program(input) => {
				debug!("read program: {:?}", input);
				match handle_input(&mut data, input) {
					Ok(s) => (s, true),
					Err(s) => (s, false),
				}
			}
			InputResult::Eof => return Err(()),
			InputResult::InputError(err) => (err, false),
			_ => (String::new(), false),
		};
		Ok(Repl {
			state: Print { to_print, success },
			data: data,
		})
	}
}

impl<'data> Repl<'data, ManualPrint> {
	/// asdfdsa
	pub fn print(self, result_output: &str, as_out: bool) -> Repl<'data, Print> {
		let to_print = result_output.to_string();
		let success = as_out;
		Repl {
			state: Print { to_print, success },
			data: self.data,
		}
	}
}

impl<'d> Repl<'d, Print> {
	/// Prints the result if successful as `[out#]` or the failure message if any.
	pub fn print<W: io::Write>(self, wtr: &mut W) -> Repl<'d, Read> {
		let Repl { state, data } = self;
		if state.success {
			if data.statements.len() > 0 {
				let out_stmt = format!("[out{}]", data.statements.len() - 1);
				writeln!(
					wtr,
					"{} {}: {}",
					data.name.color(data.prompt_colour),
					out_stmt.color(data.out_colour),
					state.to_print
				)
				.expect("failed writing");
			}
		} else {
			writeln!(wtr, "{}", state.to_print).expect("failed writing");
		}

		Repl {
			state: Read,
			data: data,
		}
	}
}

/// Runs a single program input.
fn handle_input(data: &mut ReplData, input: Input) -> Result<String, String> {
	let additionals = build_additionals(input, data.statements.len());
	let src = build_source(data, additionals.clone());
	match eval(&compile_dir(), src) {
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
			if let Some(stmts) = additionals.stmts {
				data.statements.push(stmts.stmts);
			}
			Ok(s)
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

fn build_source(data: &mut ReplData, additional: Additional) -> SourceFile {
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
fn eval<P: AsRef<Path>>(compile_dir: P, source: SourceFile) -> Result<String, String> {
	let mut c = Exe::compile(&source, compile_dir).unwrap();

	let compilation_stderr = {
		// output stderr stream line by line, erasing each line as you go.
		let rdr = BufReader::new(c.stderr());
		let mut s = String::new();
		for line in rdr.lines() {
			let line = line.unwrap();
			overwrite_current_console_line(&line);
			s.push_str(&line);
			s.push('\n');
		}
		overwrite_current_console_line("");
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
							println!("{}", first);
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
