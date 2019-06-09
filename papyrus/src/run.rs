#[cfg(feature = "racer-completion")]
use crate::complete::code::CodeCompleter;
use crate::complete::{cmdr::TreeCompleter, modules::ModulesCompleter};
use crate::output;
use crate::prelude::*;
#[cfg(feature = "racer-completion")]
use linefeed::Suffix;
use linefeed::Terminal;
use linefeed::{Completion, DefaultTerminal, Interface, Prompter};
use repl::{Read, ReadResult, Signal};
use std::cmp::max;
use std::io;
use std::sync::Arc;

impl<D> Repl<Read, D> {
    /// Run the REPL interactively.
    /// Consumes the REPL in the process and will block this thread until exited.
    pub fn run(self, app_data: &mut D) -> io::Result<()> {
        self.run_inner(app_data, false)
    }

    /// Run the REPL interactively.
    /// Consumes the REPL in the process and will block this thread until exited.
    /// Racer code completion is enabled.
    #[cfg(feature = "racer-completion")]
    pub fn run_with_racer_completion(self, app_data: &mut D) -> io::Result<()> {
        self.run_inner(app_data, true)
    }

    fn run_inner(self, app_data: &mut D, racer: bool) -> io::Result<()> {
        cratesiover::output_to_writer(
            "papyrus",
            env!("CARGO_PKG_VERSION"),
            &mut std::io::stdout(),
        )?;

        let def_term = DefaultTerminal::new()?;
        let term = Interface::new("papyrus")?;

        let mut read = self;

        loop {
            term.set_prompt(&read.prompt())?;

            let completer = Completer::build(&read.data, racer);
            term.set_completer(Arc::new(completer));

            let input = match term.read_line()? {
                linefeed::ReadResult::Input(s) => s,
                _ => String::new(),
            };

            read.line_input(&input);

            if !input.is_empty() {
                term.add_history_unique(input);
            }

            match read.read() {
                ReadResult::Read(repl) => read = repl,
                ReadResult::Eval(mut repl) => {
                    // output to stdout
                    let rx = repl.output_listen();
                    let jh = std::thread::spawn(move || output_repl(rx));

                    let result = repl.eval(app_data);

                    match result.signal {
                        Signal::None => (),
                        Signal::Exit => break,
                    }

                    read = result.repl.print();

                    read.close_channel();

                    jh.join().ok().unwrap()?; // wait to finish printing

                    // erase last line, otherwise a double prompt will be set
                    def_term.lock_write().move_to_first_column()?;
                }
            }
        }

        Ok(())
    }
}

fn output_repl(rx: output::Receiver) -> std::io::Result<()> {
    use term_cursor as cursor;

    let term = DefaultTerminal::new()?;

    let mut pos = cursor::get_pos().unwrap_or((0, 0));

    for msg in rx.iter() {
        match msg {
            output::OutputChange::CurrentLine(line) => {
                let mut o = term.lock_write();

                let p = cursor::get_pos().unwrap_or((0, 0));

                let diff = (p.1 as usize).saturating_sub(pos.1 as usize);

                o.move_up(diff)?;
                o.move_to_first_column()?;
                o.clear_to_screen_end()?;

                o.write(&line)?;

                o.flush()?;
            }
            output::OutputChange::NewLine(line) => {
                let mut o = term.lock_write();

                o.write("\n")?;

                pos = cursor::get_pos().unwrap_or((0, 0));

                o.write(&line)?;

                o.flush()?;
            }
        }
    }

    Ok(())
}

struct Completer {
    tree_cmplter: TreeCompleter,
    mod_cmplter: ModulesCompleter,
    #[cfg(feature = "racer-completion")]
    code_cmplter: Option<CodeCompleter>,
}

impl Completer {
    #[cfg(feature = "racer-completion")]
    fn build<T>(rdata: &repl::ReplData<T>, racer: bool) -> Self {
        let tree_cmplter = TreeCompleter::build(&rdata.cmdtree);

        let mod_cmplter = ModulesCompleter::build(&rdata.cmdtree, rdata.file_map());

        let code_cmplter = if racer {
            Some(CodeCompleter::build(rdata))
        } else {
            None
        };

        Self {
            tree_cmplter,
            mod_cmplter,
            code_cmplter,
        }
    }

    #[cfg(not(feature = "racer-completion"))]
    fn build<T>(rdata: &repl::ReplData<T>, _racer: bool) -> Self {
        let tree_cmplter = TreeCompleter::build(&rdata.cmdtree);

        let mod_cmplter = ModulesCompleter::build(&rdata.cmdtree, rdata.file_map());

        Self {
            tree_cmplter,
            mod_cmplter,
        }
    }
}

#[cfg(not(feature = "racer-completion"))]
impl<T: Terminal> linefeed::Completer<T> for Completer {
    fn complete(
        &self,
        _word: &str,
        prompter: &Prompter<T>,
        _start: usize,
        _end: usize,
    ) -> Option<Vec<Completion>> {
        let mut v = Vec::new();

        let line = prompter.buffer();

        let trees = self
            .tree_cmplter
            .complete(line)
            .map(|x| Completion::simple(x.to_string()));
        v.extend(trees);

        let mods = self
            .mod_cmplter
            .complete(line)
            .map(|x| x.map(|y| Completion::simple(y)));
        if let Some(mods) = mods {
            v.extend(mods);
        }

        if v.len() > 0 {
            Some(v)
        } else {
            None
        }
    }

    fn word_start(&self, line: &str, _end: usize, _prompter: &Prompter<T>) -> usize {
        let s1 = TreeCompleter::word_break(line);
        let s2 = ModulesCompleter::word_break(line);

        max(s1, s2)
    }
}

#[cfg(feature = "racer-completion")]
impl<T: Terminal> linefeed::Completer<T> for Completer {
    fn complete(
        &self,
        _word: &str,
        prompter: &Prompter<T>,
        _start: usize,
        _end: usize,
    ) -> Option<Vec<Completion>> {
        let mut v = Vec::new();

        let line = prompter.buffer();

        let trees = self
            .tree_cmplter
            .complete(line)
            .map(|x| Completion::simple(x.to_string()));
        v.extend(trees);

        let mods = self
            .mod_cmplter
            .complete(line)
            .map(|x| x.map(|y| Completion::simple(y)));
        if let Some(mods) = mods {
            v.extend(mods);
        }

        if !line.starts_with('.') {
            let code = self.code_cmplter.as_ref().map(|x| {
                x.complete(line, Some(10)).into_iter().map(|x| Completion {
                    completion: x.matchstr,
                    display: None,
                    suffix: Suffix::None,
                })
            });
            if let Some(code) = code {
                v.extend(code);
            }
        }

        if v.len() > 0 {
            Some(v)
        } else {
            None
        }
    }

    fn word_start(&self, line: &str, _end: usize, _prompter: &Prompter<T>) -> usize {
        let s1 = TreeCompleter::word_break(line);
        let s2 = ModulesCompleter::word_break(line);
        let s3 = CodeCompleter::word_start(line);

        max(max(s1, s2), s3)
    }
}
