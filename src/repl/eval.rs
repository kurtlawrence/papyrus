use super::command::Commands;
use super::*;
use linefeed::terminal::Terminal;
use pfh::{self, Input};
use std::path::Path;

type HandleInputResult = (String, bool);

enum CommonResult<'data, Term: Terminal, Data> {
    Handled(Result<Repl<'data, Print, Term, Data>, ()>),
    Program((pfh::Input, &'data mut ReplData<Data>, ReplTerminal<Term>)),
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
        InputResult::Command(cmds) => {
            debug!("read command: {}", cmds);
            unimplemented!();
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
    pub fn eval(self, app_data: Data) -> Result<Repl<'data, Print, Term, Data>, EvalSignal> {
        let Repl {
            state,
            terminal,
            data,
        } = self;

        let (to_print, as_out) = match state.result {
            InputResult::Command(cmds) => data.handle_command(&cmds, &terminal.terminal)?,
            InputResult::Program(input) => data.handle_program(input, &terminal.terminal, app_data),
            InputResult::InputError(err) => (err, false),
            InputResult::Eof => return Err(EvalSignal::Exit),
            _ => (String::new(), false),
        };

        Ok(Repl {
            state: Print { to_print, as_out },
            terminal: terminal,
            data: data,
        })
    }
}

impl<Data> ReplData<Data> {
    fn handle_command<T: Terminal>(&mut self, cmds: &str, terminal: &T) -> Result<HandleInputResult, EvalSignal>  {
        use cmdtree::LineResult as lr;

        let tuple = match self.cmdtree.parse_line(cmds, true, &mut Writer(terminal)) {
            lr::Exit => return Err(EvalSignal::Exit),
            lr::Action(res) => match res {
                CommandResult::CancelInput => ("cancelled input".to_string(), false),
            },
            _ => (String::new(), false),
        };

		Ok(tuple)
    }

    fn handle_program<T: Terminal>(
        &mut self,
        input: Input,
        terminal: &T,
        app_data: Data,
    ) -> HandleInputResult {
        let pop_input = |repl_data: &mut ReplData<_>| {
            repl_data.get_current_file_mut().contents.pop();
        };

        let has_stmts = input.stmts.len() > 0;

        // add input file
        {
            self.get_current_file_mut().contents.push(input);
        }

        // build directory
        let res = pfh::compile::build_compile_dir(
            &self.compilation_dir,
            self.file_map.values(),
            &self.linking,
        );
        if let Err(e) = res {
            pop_input(self); // failed so don't save
            return (format!("failed to build compile directory: {}", e), false);
        }

        // format
        pfh::compile::fmt(&self.compilation_dir);

        // compile
        let lib_file = pfh::compile::compile(&self.compilation_dir, &self.linking, |line| {
            Writer(terminal)
                .overwrite_current_console_line(&line)
                .unwrap()
        });
        Writer(terminal).overwrite_current_console_line("").unwrap();
        let lib_file = match lib_file {
            Ok(f) => f,
            Err(e) => {
                pop_input(self); // failed so don't save
                return (format!("{}", e), false);
            }
        };

        if has_stmts {
            // execute
            let exec_res = {
                let current_file = self.get_current_file_mut();
                pfh::compile::exec(
                    &lib_file,
                    &pfh::eval_fn_name(&current_file.mod_path),
                    app_data,
                )
            };
            match exec_res {
                Ok(s) => ((s, true)),
                Err(e) => {
                    pop_input(self); // failed so don't save
                    (e.to_string(), false)
                }
            }
        } else {
            (String::new(), false) // do not execute if no extra statements have been added
        }
    }

    fn get_current_file_mut(&mut self) -> &mut SourceFile {
        self.file_map.get_mut(&self.current_file).expect(&format!(
            "file map does not have key: {}",
            self.current_file.display()
        ))
    }
}
