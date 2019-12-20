use super::*;
use crate::{
    cmds::{self, CommandResult},
    code::{self, Input, SourceCode, StmtGrp},
    compile,
};
use std::borrow::{Borrow, BorrowMut};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

impl<D> Repl<Evaluate, D> {
    /// Evaluates the read input, compiling and executing the code and printing all line prints until
    /// a result is found. This result gets passed back as a print ready repl.
    pub fn eval(self, app_data: &mut D) -> EvalResult<D> {
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

    /// Begin listening to line change events on the output.
    pub fn output_listen(&mut self) -> output::Receiver {
        self.state.output.listen()
    }

    /// Close the sender side of the output channel.
    pub fn close_channel(&mut self) {
        self.state.output.close()
    }

    /// The current output.
    ///
    /// The output contains colouring ANSI escape codes, the prompt, and all input.
    pub fn output(&self) -> &str {
        self.state.output.buffer()
    }
}

impl<D: 'static + Send> Repl<Evaluate, D> {
    /// Same as `eval` but will evaluate on another thread, not blocking this one.
    ///
    /// An `Arc::clone` will be taken of `app_data`.
    pub fn eval_async(self, app_data: &Arc<Mutex<D>>) -> Evaluating<D> {
        let (tx, rx) = crossbeam_channel::bounded(1);

        let clone = Arc::clone(app_data);

        std::thread::spawn(move || {
            let eval = map_variants(
                self,
                || clone.lock().expect("failed getting lock of data"),
                || clone.lock().expect("failed getting lock of data"),
            );

            tx.send(eval).unwrap();
        });

        Evaluating { jh: rx }
    }
}

impl<D> Evaluating<D> {
    /// Evaluating has finished.
    pub fn completed(&self) -> bool {
        !self.jh.is_empty()
    }

    /// Waits for the evaluating to finish before return the result.
    /// If evaluating is `completed` this will return immediately.
    pub fn wait(self) -> EvalResult<D> {
        self.jh
            .recv()
            .expect("receiving eval result from async thread failed")
    }
}

fn map_variants<D, Fmut, Fbrw, Rmut, Rbrw>(
    repl: Repl<Evaluate, D>,
    obtain_mut_data: Fmut,
    obtain_brw_data: Fbrw,
) -> EvalResult<D>
where
    Fmut: FnOnce() -> Rmut,
    Rmut: DerefMut<Target = D>,
    Fbrw: FnOnce() -> Rbrw,
    Rbrw: Deref<Target = D>,
{
    let Repl {
        state,
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
            let r = data.handle_command(&cmds, &mut output, obtain_mut_data);
            keep_mutating = data.linking.mutable; // a command can alter the mutating state, needs to persist
            r.map(|x| EvalOutput::Print(x))
        }
        InputResult::Program(input) => {
            Ok(data.handle_program(input, &mut output, obtain_mut_data, obtain_brw_data))
        }
        InputResult::InputError(err) => Ok(EvalOutput::Print(Cow::Owned(err))),
        InputResult::Eof => Err(Signal::Exit),
        _ => Ok(EvalOutput::Print(Cow::Borrowed(""))),
    };

    let (eval_output, sig) = match mapped {
        Ok(hir) => (hir, Signal::None),
        Err(sig) => (EvalOutput::Print(Cow::Borrowed("")), sig),
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
                data: eval_output,
            },
            data,
            more,
            data_mrker,
        },
    }
}

impl<D> ReplData<D> {
    fn handle_command<F, R, W>(
        &mut self,
        cmds: &str,
        writer: &mut W,
        obtain_mut_app_data: F,
    ) -> Result<Cow<'static, str>, Signal>
    where
        F: FnOnce() -> R,
        R: DerefMut<Target = D>,
        W: io::Write,
    {
        use cmdtree::LineResult as lr;

        let tuple = match self.cmdtree.parse_line(cmds, true, writer) {
            lr::Exit => return Err(Signal::Exit),
            lr::Cancel => {
                self.linking.mutable = false; // reset the mutating on cancel
                self.editing = None; // reset the editing on cancel
                Cow::Borrowed("cancelled input and returned to root")
            }
            lr::Action(res) => match res {
                CommandResult::BeginMutBlock => {
                    self.linking.mutable = true;
                    Cow::Borrowed("beginning mut block")
                }
                CommandResult::EditAlter(ei) => Cow::Borrowed(cmds::edit_alter(self, ei)),
                CommandResult::EditReplace(ei, val) => {
                    let r = Cow::Borrowed(cmds::edit_alter(self, ei));

                    if r.is_empty() {
                        return Err(Signal::ReEvaluate(val));
                    } else {
                        r
                    }
                }
                CommandResult::SwitchModule(path) => {
                    Cow::Borrowed(crate::cmds::switch_module(self, &path))
                }

                CommandResult::ActionOnReplData(action) => Cow::Owned(action(self, writer)),
                CommandResult::ActionOnAppData(action) => {
                    let mut r = obtain_mut_app_data();
                    let app_data: &mut D = r.borrow_mut();
                    let s = action(app_data, writer);
                    Cow::Owned(s)
                }
                CommandResult::Empty => Cow::Borrowed(""),
            },
            _ => Cow::Borrowed(""),
        };

        Ok(tuple)
    }

    fn handle_program<Fmut, Fbrw, Rmut, Rbrw>(
        &mut self,
        mut input: Input,
        writer: &mut Output<output::Write>,
        obtain_mut_data: Fmut,
        obtain_brw_data: Fbrw,
    ) -> EvalOutput
    where
        Fmut: FnOnce() -> Rmut,
        Rmut: DerefMut<Target = D>,
        Fbrw: FnOnce() -> Rbrw,
        Rbrw: Deref<Target = D>,
    {
        let (nitems, ncrates) = (input.items.len(), input.crates.len());

        let has_stmts = input.stmts.len() > 0;

        let (lstmts, litem, lcrates) = {
            let src = self.current_src();
            (src.stmts.len(), src.items.len(), src.crates.len())
        };

        let mut undo = true;

        let (stmt_idx, item_idx, crate_idx) = if let Some(ei) = self.editing.take() {
            let src = self.get_current_file_mut(); // remove at the index
                                                   // then insert, so
                                                   // acts like replace

            undo = false;

            match ei.editing {
                // we clear the edits if the indices fall outside the bounds
                Editing::Stmt => {
                    if ei.index >= lstmts {
                        input.stmts.clear();
                    } else {
                        src.stmts.remove(ei.index);
                    }

                    (ei.index, litem, lcrates)
                }
                Editing::Item => {
                    if ei.index >= litem {
                        input.items.clear();
                    } else {
                        src.items.remove(ei.index);
                    }

                    (lstmts, ei.index, lcrates)
                }
                Editing::Crate => {
                    if ei.index >= lcrates {
                        input.crates.clear();
                    } else {
                        src.crates.remove(ei.index);
                    }

                    (lstmts, litem, ei.index)
                }
            }
        } else {
            (lstmts, litem, lcrates)
        };

        self.insert_input(input, stmt_idx, item_idx, crate_idx);

        let maybe_pop_input = |repl_data: &mut ReplData<D>| {
            if undo {
                let src = repl_data.get_current_file_mut();

                if has_stmts {
                    src.stmts.remove(stmt_idx);
                }

                for _ in 0..nitems {
                    src.items.remove(item_idx);
                }

                for _ in 0..ncrates {
                    src.crates.remove(crate_idx);
                }
            }
        };

        // build directory
        let res = compile::build_compile_dir(&self.compilation_dir, &self.mods_map, &self.linking);
        if let Err(e) = res {
            maybe_pop_input(self); // failed so don't save
            return EvalOutput::Print(Cow::Owned(format!(
                "failed to build compile directory: {}",
                e
            )));
        }

        // compile
        let lib_file = compile::compile(&self.compilation_dir, &self.linking, |line| {
            writer.erase_last_line();
            writer.write_str(line);
        });

        writer.erase_last_line();

        let lib_file = match lib_file {
            Ok(f) => f,
            Err(e) => {
                maybe_pop_input(self); // failed so don't save
                return EvalOutput::Print(Cow::Owned(format!("{}", e)));
            }
        };

        if has_stmts {
            // execute
            let exec_res = {
                // once compilation succeeds and we are going to evaluate it (which libloads) we
                // first rename the files to avoid locking for the next compilation that might
                // happen
                let lib_file = compile::unshackle_library_file(lib_file);

                let mut fn_name = String::new();
                code::eval_fn_name(&code::into_mod_path_vec(self.current_mod()), &mut fn_name);

                if self.linking.mutable {
                    let mut r = obtain_mut_data();
                    let app_data: &mut D = r.borrow_mut();
                    compile::exec(&lib_file, &fn_name, app_data)
                } else {
                    let r = obtain_brw_data();
                    let app_data: &D = r.borrow();
                    compile::exec(&lib_file, &fn_name, app_data)
                }
            };
            match exec_res {
                Ok((kserd, lib)) => {
                    // store vec, maybe
                    add_to_limit_vec(&mut self.loadedlibs, lib, self.loaded_libs_size_limit);

                    if self.linking.mutable {
                        maybe_pop_input(self); // don't save mutating inputs
                        EvalOutput::Print(Cow::Owned(format!("finished mutating block: {}", kserd)))
                    // don't print as `out#`
                    } else {
                        EvalOutput::Data(kserd)
                    }
                }
                Err(e) => {
                    maybe_pop_input(self); // failed so don't save
                    EvalOutput::Print(Cow::Borrowed(e))
                }
            }
        } else {
            // this will keep inputs, might not be preferrable to do so in mutating state?
            EvalOutput::Print(Cow::Borrowed("")) // do not execute if no extra statements have been added
        }
    }

    fn insert_input(&mut self, input: Input, stmt_idx: usize, item_idx: usize, crate_idx: usize) {
        let Input {
            items,
            crates,
            stmts,
        } = input;

        let src = self.get_current_file_mut();

        if !stmts.is_empty() {
            src.stmts.insert(stmt_idx, StmtGrp(stmts));
        }

        for item in items.into_iter().rev() {
            src.items.insert(item_idx, item);
        }

        for cr in crates.into_iter().rev() {
            src.crates.insert(crate_idx, cr);
        }
    }

    fn get_current_file_mut(&mut self) -> &mut SourceCode {
        self.mods_map.get_mut(&self.current_mod).expect(&format!(
            "file map does not have key: {}",
            self.current_mod.display()
        ))
    }
}

fn add_to_limit_vec<T>(store: &mut VecDeque<T>, item: T, limit: usize) {
    match (limit, store.len()) {
        (0, 0) => (),             // do nothing, lib will drop after this
        (0, _x) => store.clear(), // zero limit and store has something, clear them
        (limit, _) => {
            let limit = limit - 1; // limit will be gt zero
            store.truncate(limit); // truncate to limit - 1 length, as we will add new lib in
            store.push_front(item); // we keep the newest versions at front of queue
        }
    }
}

#[test]
fn vec_limited_testing() {
    let mut vec: VecDeque<i32> = VecDeque::new();
    vec.push_front(-3);
    vec.push_front(-2);

    add_to_limit_vec(&mut vec, 0, 3);
    assert_eq!(&vec, &[0, -2, -3]);

    add_to_limit_vec(&mut vec, 3, 3);
    assert_eq!(&vec, &[3, 0, -2]);

    add_to_limit_vec(&mut vec, -1, 0);
    assert!(vec.is_empty());
    add_to_limit_vec(&mut vec, -1, 0);
    assert!(vec.is_empty());

    add_to_limit_vec(&mut vec, 0, 1);
    add_to_limit_vec(&mut vec, 1, 1);
    add_to_limit_vec(&mut vec, 2, 1);
    assert_eq!(&vec, &[2]);
}
