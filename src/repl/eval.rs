use super::command::Commands;
use super::*;
use linefeed::terminal::Terminal;
use pfh::{self, Input};
use std::path::Path;

type HandleInputResult = (String, bool);

enum CommonResult<'data, Term: Terminal, Data> {
    Handled(Result<Repl<'data, Print, Term, Data>, ()>),
    Program(
        (
            pfh::Input,
            &'data mut ReplData<Term, Data>,
            ReplTerminal<Term>,
        ),
    ),
}

/// bit dumb but i have to extract out the common code otherwise i will have code maintenance hell
/// the other code returns an Ok(Result<Print, ()>) and the program arm returns Err((input, data, terminal)) such that the input processing has already been processed.
fn handle_common<'data, Term: Terminal, Data>(
    repl: Repl<'data, Evaluate, Term, Data>,
) -> CommonResult<Term, Data> {
    let Repl {
        state,
        terminal,
        data,
    } = repl;

    let (to_print, as_out) = match state.result {
        InputResult::Command(name, args) => {
            debug!("read command: {} {:?}", name, args);
            match data.commands.find_command(&name) {
                Err(e) => (e.to_string(), false),
                Ok(cmd) => {
                    return CommonResult::Handled((cmd.action)(
                        Repl {
                            state: ManualPrint,
                            terminal: terminal,
                            data: data,
                        },
                        &args,
                    ));
                }
            }
        }
        InputResult::Program(input) => {
            return CommonResult::Program((input, data, terminal));
        }
        InputResult::Eof => return CommonResult::Handled(Err(())),
        InputResult::InputError(err) => (err, false),
        _ => (String::new(), false),
    };
    CommonResult::Handled(Ok(Repl {
        state: Print { to_print, as_out },
        terminal: terminal,
        data: data,
    }))
}

impl<'data, Term: Terminal, Data> Repl<'data, Evaluate, Term, Data> {
    /// Evaluates the read input, compiling and executing the code and printing all line prints until a result is found.
    /// This result gets passed back as a print ready repl.
    /// Does not transfer any app data as configured.
    pub fn eval(self, app_data: Data) -> Result<Repl<'data, Print, Term, Data>, ()> {
        match handle_common(self) {
            CommonResult::Handled(r) => r,
            CommonResult::Program((input, mut data, terminal)) => {
                debug!("read program: {:?}", input);
                let (to_print, as_out) =
                    match handle_program(&mut data, input, &terminal.terminal, app_data) {
                        Ok((s, as_out)) => (s, as_out),
                        Err(s) => (s, false),
                    };

                Ok(Repl {
                    state: Print { to_print, as_out },
                    terminal: terminal,
                    data: data,
                })
            }
        }
    }
}

/// Runs a single program input.
fn handle_program<T, Data>(
    data: &mut ReplData<T, Data>,
    input: Input,
    terminal: &T,
    app_data: Data,
) -> Result<HandleInputResult, String>
where
    T: Terminal,
{
    let pop_input = |repl_data| {
        get_current_file_mut(repl_data).contents.pop();
    };

    let has_stmts = input.stmts.len() > 0;

    // add input file
    {
        get_current_file_mut(data).contents.push(input);
    }

    // build directory
    let res = pfh::compile::build_compile_dir(
        &data.compilation_dir,
        data.file_map.values(),
        data.linking.as_ref(),
    );
    if let Err(e) = res {
        pop_input(data);
        return Err(format!("failed to build compile directory: {}", e));
    }

    // format
    pfh::compile::fmt(&data.compilation_dir);

    // compile
    let lib_file = pfh::compile::compile(&data.compilation_dir, data.linking.as_ref(), |line| {
        Writer(terminal)
            .overwrite_current_console_line(&line)
            .unwrap()
    });
    Writer(terminal).overwrite_current_console_line("").unwrap();
    let lib_file = match lib_file {
        Ok(f) => f,
        Err(e) => {
            pop_input(data);
            return Err(format!("{}", e));
        }
    };

    if has_stmts {
        // execute
        let exec_res = {
            let current_file = get_current_file_mut(data);
            pfh::compile::exec(
                &lib_file,
                &pfh::eval_fn_name(&current_file.mod_path),
                app_data,
            )
        };
        match exec_res {
            Ok(s) => Ok((s, true)),
            Err(e) => {
                pop_input(data);
                Err(e.to_string())
            }
        }
    } else {
        Ok((String::new(), false)) // do not execute if no extra statements have been added
    }
}

fn get_current_file_mut<T, Data>(data: &mut ReplData<T, Data>) -> &mut SourceFile
where
    T: Terminal,
{
    data.file_map.get_mut(&data.current_file).expect(&format!(
        "file map does not have key: {}",
        data.current_file.display()
    ))
}
