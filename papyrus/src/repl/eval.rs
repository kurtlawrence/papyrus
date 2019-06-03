use super::*;
use crate::compile;
use crate::pfh::{self, Input, StmtGrp};
use linefeed::terminal::Terminal;
use std::borrow::{Borrow, BorrowMut};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Represents a type of `(to_print, as_out)`.
/// `as_out` flags to output `out#`.
type HandleInputResult = (Cow<'static, str>, bool);

impl<Term: Terminal, Data> Repl<Evaluate, Term, Data> {
    /// Evaluates the read input, compiling and executing the code and printing all line prints until
    /// a result is found. This result gets passed back as a print ready repl.
    pub fn eval(self, app_data: &mut Data) -> EvalResult<Term, Data> {
        use std::cell::Cell;
        use std::rc::Rc;

        let ptr = Rc::into_raw(Rc::new(app_data));

        // as I am playing around with pointers here, I am going to do assertions in the rebuilding
        // if from_raw is called more than once, it is memory unsafe, so count the calls and assert it is only 1
        let rebuilds: Rc<Cell<u32>> = Rc::new(Cell::new(0));

        let func = || {
            let b = Rc::clone(&rebuilds);

            let n = b.get();

            assert_eq!(n, 0, "unsafe memory operation, can only rebuild Rc once.");

            b.set(n + 1);

            let c = unsafe { Rc::from_raw(ptr) };

            Rc::try_unwrap(c)
                .map_err(|_| "there should only be one strong reference")
                .unwrap()
        };

        map_variants(self, func, func)
    }
}

impl<Term: Terminal + 'static, Data: 'static + Send + Sync> Repl<Evaluate, Term, Data> {
    /// Same as `eval` but will evaluate on another thread, not blocking this one.
    ///
    /// An `Arc::clone` will be taken of `app_data`. `RwLock` generally takes a read
    /// lock, making it possible to take more read locks in another thread. A write lock
    /// will be taken when required, currently when in a mutating block or a command action
    /// is invoked.
    ///
    /// > Be careful of blocking a program by taking a read lock and calling this function
    /// when a write lock is required.
    pub fn eval_async(self, app_data: &Arc<RwLock<Data>>) -> Evaluating<Term, Data> {
        let (tx, rx) = crossbeam_channel::bounded(1);

        let clone = Arc::clone(app_data);

        std::thread::spawn(move || {
            let eval = map_variants(
                self,
                || clone.write().expect("failed getting write lock of data"),
                || clone.read().expect("failed getting read lock of data"),
            );

            tx.send(eval).unwrap();
        });

        Evaluating { jh: rx }
    }
}

impl<Term: Terminal, Data> Evaluating<Term, Data> {
    /// Evaluating has finished.
    pub fn completed(&self) -> bool {
        !self.jh.is_empty()
    }

    /// Waits for the evaluating to finish before return the result.
    /// If evaluating is `completed` this will return immediately.
    pub fn wait(self) -> EvalResult<Term, Data> {
        self.jh
            .recv()
            .expect("receiving eval result from async thread failed")
    }
}

fn map_variants<T, D, Fmut, Fbrw, Rmut, Rbrw>(
    repl: Repl<Evaluate, T, D>,
    obtain_mut_data: Fmut,
    obtain_brw_data: Fbrw,
) -> EvalResult<T, D>
where
    T: Terminal,
    Fmut: FnOnce() -> Rmut,
    Rmut: DerefMut<Target = D>,
    Fbrw: FnOnce() -> Rbrw,
    Rbrw: Deref<Target = D>,
{
    let Repl {
        state,
        terminal,
        mut data,
        more,
        data_mrker,
    } = repl;

    let Evaluate { mut output, result } = state;

    let mut keep_mutating = false; // default to stop mutating phase
                                   // can't cancel before as handle program requires it for decisions

    // map variants into Result<HandleInputResult, EvalSignal>
    let mapped = match result {
        InputResult::Command(cmds) => {
            let r = data.handle_command(&cmds, &terminal.terminal, &mut output, obtain_mut_data);
            keep_mutating = data.linking.mutable; // a command can alter the mutating state, needs to persist
            r
        }
        InputResult::Program(input) => Ok(data.handle_program(
            input,
            &terminal.terminal,
            &mut output,
            obtain_mut_data,
            obtain_brw_data,
        )),
        InputResult::InputError(err) => Ok((Cow::Owned(err), false)),
        InputResult::Eof => Err(Signal::Exit),
        _ => Ok((Cow::Borrowed(""), false)),
    };

    let ((to_print, as_out), sig) = match mapped {
        Ok(hir) => (hir, Signal::None),
        Err(sig) => ((Cow::Borrowed(""), false), sig),
    };

    data.linking.mutable = keep_mutating; // always cancel a mutating block on evaluation??
                                          // the alternative would be to keep alive on compilation failures, might not for now though.
                                          // this would have to be individually handled in each match arm and it, rather let the user
                                          // have to reinstate mutability if they fuck up input.

    EvalResult {
        signal: sig,
        repl: Repl {
            state: Print {
                output,
                to_print,
                as_out,
            },
            terminal,
            data,
            more,
            data_mrker,
        },
    }
}

impl<Data> ReplData<Data> {
    fn handle_command<T, F, R, W>(
        &mut self,
        cmds: &str,
        terminal: &Arc<T>,
        writer: &mut W,
        obtain_mut_app_data: F,
    ) -> Result<HandleInputResult, Signal>
    where
        T: Terminal,
        F: FnOnce() -> R,
        R: DerefMut<Target = Data>,
        W: io::Write,
    {
        use cmdtree::LineResult as lr;

        let tuple = match self.cmdtree.parse_line(cmds, true, writer) {
            lr::Exit => return Err(Signal::Exit),
            lr::Cancel => {
                self.linking.mutable = false; // reset the mutating on cancel
                (Cow::Borrowed("cancelled input and returned to root"), false)
            }
            lr::Action(res) => match res {
                CommandResult::BeginMutBlock => {
                    self.linking.mutable = true;
                    (Cow::Borrowed("beginning mut block"), false)
                }
                CommandResult::ActionOnReplData(action) => {
                    let s = action.call_box((self, writer));
                    (Cow::Owned(s), false)
                }
                CommandResult::ActionOnAppData(action) => {
                    let mut r = obtain_mut_app_data();
                    let app_data: &mut Data = r.borrow_mut();
                    let s = action.call_box((app_data, writer));
                    (Cow::Owned(s), false)
                }
                CommandResult::Empty => (Cow::Borrowed(""), false),
            },
            _ => (Cow::Borrowed(""), false),
        };

        Ok(tuple)
    }

    fn handle_program<T, Fmut, Fbrw, Rmut, Rbrw>(
        &mut self,
        input: Input,
        terminal: &Arc<T>,
        writer: &mut Output<output::Write>,
        obtain_mut_data: Fmut,
        obtain_brw_data: Fbrw,
    ) -> HandleInputResult
    where
        T: Terminal,
        Fmut: FnOnce() -> Rmut,
        Rmut: DerefMut<Target = Data>,
        Fbrw: FnOnce() -> Rbrw,
        Rbrw: Deref<Target = Data>,
    {
        let (nitems, ncrates) = (input.items.len(), input.crates.len());

        let has_stmts = input.stmts.len() > 0;

        let pop_input = |repl_data: &mut ReplData<Data>| {
            let src = repl_data.get_current_file_mut();
            src.items.truncate(src.items.len() - nitems);
            src.crates.truncate(src.crates.len() - ncrates);
            if has_stmts {
                src.stmts.pop();
            }
        };

        // add input file
        {
            let Input {
                items,
                crates,
                stmts,
            } = input;

            let src = self.get_current_file_mut();

            src.items.extend(items);
            src.crates.extend(crates);
            if has_stmts {
                src.stmts.push(StmtGrp(stmts))
            }
        }

        // build directory
        let res = compile::build_compile_dir(&self.compilation_dir, &self.file_map, &self.linking);
        if let Err(e) = res {
            pop_input(self); // failed so don't save
            return (
                Cow::Owned(format!("failed to build compile directory: {}", e)),
                false,
            );
        }

        // format
        compile::fmt(&self.compilation_dir);

        // compile
        let lib_file = compile::compile(&self.compilation_dir, &self.linking, |line| {
            writer.erase_last_line();
            writer.write_str(line);
        });

        writer.erase_last_line();

        let lib_file = match lib_file {
            Ok(f) => f,
            Err(e) => {
                pop_input(self); // failed so don't save
                return (Cow::Owned(format!("{}", e)), false);
            }
        };

        dbg!(writer);

        if has_stmts {
            // execute
            let exec_res = {
                // Has to be done to make linux builds work
                // see:
                //		https://github.com/nagisa/rust_libloading/issues/5
                //		https://github.com/nagisa/rust_libloading/issues/41
                //		https://github.com/nagisa/rust_libloading/issues/49
                //
                // Basically the api function `dlopen` will keep loaded libraries in memory to avoid
                // continuously allocating memory. It only does not release the library when thread_local data
                // is hanging around, and it seems `println!()` is something that does this.
                // Hence to avoid not having the library not updated with a new `new()` call, a different lib
                // name is passed to the function.
                // This is very annoying as it has needless fs interactions and a growing fs footprint but
                // what can you do ¯\_(ツ)_/¯
                let lib_file = rename_lib_file(lib_file).expect("failed renaming library file");

                let redirect_wtr = if self.redirect_on_execution {
                    Some(OwnedWriter(Arc::clone(terminal)))
                } else {
                    None
                };

                let mut fn_name = String::new();
                pfh::eval_fn_name(&pfh::into_mod_path_vec(&self.current_file), &mut fn_name);

                if self.linking.mutable {
                    let mut r = obtain_mut_data();
                    let app_data: &mut Data = r.borrow_mut();
                    compile::exec(&lib_file, &fn_name, app_data, redirect_wtr)
                } else {
                    let r = obtain_brw_data();
                    let app_data: &Data = r.borrow();
                    compile::exec(&lib_file, &fn_name, app_data, redirect_wtr)
                }
            };
            match exec_res {
                Ok(s) => {
                    if self.linking.mutable {
                        pop_input(self); // don't save mutating inputs
                        ((Cow::Owned(format!("finished mutating block: {}", s)), false)) // don't print as `out#`
                    } else {
                        ((Cow::Owned(s), true))
                    }
                }
                Err(e) => {
                    pop_input(self); // failed so don't save
                    (Cow::Borrowed(e), false)
                }
            }
        } else {
            // this will keep inputs, might not be preferrable to do so in mutating state?
            (Cow::Borrowed(""), false) // do not execute if no extra statements have been added
        }
    }

    fn get_current_file_mut(&mut self) -> &mut pfh::SourceCode {
        self.file_map.get_mut(&self.current_file).expect(&format!(
            "file map does not have key: {}",
            self.current_file.display()
        ))
    }
}

/// Renames the library into a distinct file name by incrementing a counter.
/// Could fail if the number of libs grows enormous, greater than `u64`. This would mean, with
/// `u64 = 18,446,744,073,709,551,615`, even with 1KB files (prolly not) this would be
/// 18,446,744,073 TB. User will probably know something is up.
fn rename_lib_file<P: AsRef<Path>>(compiled_lib: P) -> io::Result<PathBuf> {
    let no_parent = PathBuf::new();
    let mut idx: u64 = 0;
    let parent = compiled_lib.as_ref().parent().unwrap_or(&no_parent);
    let name = |i| format!("papyrus.mem-code.lib.{}", i);
    let mut lib_path = parent.join(&name(idx));
    while lib_path.exists() {
        idx += 1;
        lib_path = parent.join(&name(idx));
    }
    std::fs::rename(&compiled_lib, &lib_path)?;
    Ok(lib_path)
}
