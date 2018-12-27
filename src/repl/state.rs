use super::*;
use linefeed::terminal::Terminal;

impl<S> Repl<S> {
	pub fn load<P: AsRef<Path>>(self, file_path: P) -> Repl<Evaluate> {
		let result = load_and_parse(file_path);
		Repl {
			state: Evaluate { result },
			inner: self.inner,
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

impl Repl<Read> {
	/// A new REPL instance.
	pub fn new() -> Repl<Read> {
		let mut r = Data {
			commands: Vec::new(),
			items: Vec::new(),
			statements: Vec::new(),
			crates: Vec::new(),
			exit_loop: false,
			name: "papyrus",
			prompt_colour: Color::Cyan,
			out_colour: Color::BrightGreen,
			print: true,
		};
		// help
		r.commands.push(Command::new(
			"help",
			CmdArgs::Text,
			"Show help for commands",
			|args| {
				let (repl, arg) = { (args.repl, args.arg) };
				// colour output
				let output = repl.data.commands.build_help_response(if arg.is_empty() {
					None
				} else {
					Some(arg)
				});

				Ok(repl.print(&output))
			},
		));
		// exit
		r.commands.push(Command::new(
			"exit",
			CmdArgs::None,
			"Exit repl",
			|args| Err(()), // flag to break
		));
		// cancel
		r.commands.push(Command::new(
			"cancel",
			CmdArgs::None,
			"Cancels more input",
			|args| Ok(args.repl.print("")),
		));
		// cancel (with c)
		r.commands.push(Command::new(
			"c",
			CmdArgs::None,
			"Cancels more input",
			|args| Ok(args.repl.print("")),
		));
		// load
		r.commands.push(Command::new(
			"load",
			CmdArgs::Filename,
			"load *.rs or *.rscript as inputs",
			|args| {
				let (repl, arg) = { (args.repl, args.arg) };
				let eval = repl.load(arg);
				eval.eval()
			},
		));

		Repl {
			state: Read,
			inner: InnerData {},
			data: r,
		}
	}

	/// Reads input from the input reader until an evaluation phase can begin.
	pub fn read<Term: Terminal>(self, input_rdr: &mut InputReader<Term>) -> Repl<Evaluate> {
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
					inner: self.inner,
					data: self.data,
				};
			}
		}
	}

	/// Run the REPL interactively. Consumes the REPL in the process and will block this thread until exited.
	///
	/// # Panics
	/// - Failure to initialise `InputReader`.
	pub fn run(mut self) {
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
				Ok(r) => read = r.print(),
				Err(_) => break,
			}
		}
	}
}

impl Repl<Evaluate> {
	pub fn eval(self) -> Result<Repl<Print>, ()> {
		let Repl {
			state,
			inner,
			mut data,
		} = self;

		let to_print = match state.result {
			InputResult::Command(name, args) => {
				debug!("read command: {} {:?}", name, args);
				match data.commands.find_command(&name) {
					Err(e) => e.to_string(),
					Ok(cmd) => {
						return (cmd.action)(CommandActionArgs {
							repl: Repl {
								state: ManualPrint,
								inner: inner,
								data: data,
							},
							arg: &args,
						});
					}
				}
			}
			InputResult::Program(input) => {
				debug!("read program: {:?}", input);
				handle_input(&mut data, input).unwrap_or_else(|e| e)
			}
			InputResult::Eof => return Err(()),
			InputResult::InputError(err) => err,
			_ => String::new(),
		};
		Ok(Repl {
			state: Print { to_print },
			inner: inner,
			data: data,
		})
	}
}

impl Repl<ManualPrint> {
	pub fn print(self, text: &str) -> Repl<Print> {
		let to_print = text.to_string();
		Repl {
			state: Print { to_print },
			inner: self.inner,
			data: self.data,
		}
	}
}

impl Repl<Print> {
	pub fn print(self) -> Repl<Read> {
		println!("{}", self.state.to_print);
		Repl {
			state: Read,
			inner: self.inner,
			data: self.data,
		}
	}
}

/// Runs a single program input.
fn handle_input(data: &mut Data, input: Input) -> Result<String, String> {
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
			let mut yes = false;
			if let Some(stmts) = additionals.stmts {
				data.statements.push(stmts.stmts);
				yes = true;
			}
			if yes {
				let out_stmt = format!("[out{}]", data.statements.len() - 1);
				println!(
					"{} {}: {}",
					data.name.color(data.prompt_colour),
					out_stmt.color(data.out_colour),
					s
				);
			}
			Ok(s)
		}
		Err(s) => {
			print!("{}", s);
			io::stdout().flush().expect("flushing stdout failed");
			Err(s)
		}
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

fn build_source(data: &mut Data, additional: Additional) -> SourceFile {
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
fn eval<P: AsRef<Path>>(compile_dir: &P, source: SourceFile) -> Result<String, String> {
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
				let mut rdr = BufReader::new(c.stdout());
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
		let mut data = Repl::new().data;
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
		let mut data = Repl::new().data;
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
