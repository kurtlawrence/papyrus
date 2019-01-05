use super::command::Commands;
use super::*;
use file::SourceFileType;

use linefeed::terminal::Terminal;
use std::io::BufRead;
use std::io::Read as IoRead;

type HandleInputResult = (String, bool);

const PAPYRUS_SPLIT_PATTERN: &'static str = "<!papyrus-split>";

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

/// Runs a single program input.
fn handle_input<T>(
    data: &mut ReplData<T>,
    input: Input,
    terminal: &T,
) -> Result<HandleInputResult, String>
where
    T: Terminal,
{
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
fn eval<P, T>(compile_dir: P, source: SourceFile, terminal: &T) -> Result<String, String>
where
    P: AsRef<Path>,
    T: Terminal,
{
    let mut c = Exe::compile(&source, compile_dir).unwrap();

    let compilation_stderr = {
        // output stderr stream line by line, erasing each line as you go.
        let rdr = BufReader::new(c.stderr());
        let mut s = String::new();
        for line in rdr.lines() {
            let line = line.unwrap();
            Writer(terminal)
                .overwrite_current_console_line(&line)
                .unwrap();
            s.push_str(&line);
            s.push('\n');
        }
        Writer(terminal).overwrite_current_console_line("").unwrap();
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
                            writeln!(Writer(terminal), "{}", first).unwrap();
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

fn code(statements: &str, items: &str) -> String {
    format!(
        r#"fn main() {{
    {stmts}
}}

{items}
"#,
        stmts = statements,
        items = items
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use linefeed::memory::MemoryTerminal;

    const RS_FILES: [&'static str; 2] = ["with-crate.rs", "pwr.rs"];
    const RSCRIPT_FILES: [&'static str; 7] = [
        "expr.rscript",
        "one.rscript",
        "expr-list.rscript",
        "count_files.rscript",
        "items.rscript",
        "dir.rscript",
        "use_rand.rscript",
    ];

    #[test]
    fn load_rs_source() {
        let mut data = ReplData::<MemoryTerminal>::default();
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
                        &MemoryTerminal::new(),
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
        let mut data = ReplData::<MemoryTerminal>::default();
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
                        &MemoryTerminal::new(),
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
