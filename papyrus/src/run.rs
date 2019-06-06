use crate::complete::*;
use crate::output;
use crate::prelude::*;
use linefeed::Terminal;
use mortal::Cursor;
use repl::{Read, ReadResult, Signal};

// #[cfg(feature = "racer-completion")]
impl<Term: 'static + Terminal, Data> Repl<Read, Term, Data> {
    /// Run the REPL interactively. Consumes the REPL in the process and will block this thread until exited.
    ///
    /// # Panics
    /// - Failure to initialise `InputReader`.
    pub fn run(self, app_data: &mut Data) {
        cratesiover::output_to_writer("papyrus", env!("CARGO_PKG_VERSION"), &mut std::io::stdout())
            .unwrap();

        let mut term = mortal::Terminal::new().unwrap();

        let mut read = self;

        // output to stdout
        let rx = read.output_listen();
        std::thread::spawn(move || output_repl(rx).unwrap());

        loop {
            let combined = CombinedCompleter {
                completers: vec![
                    Box::new(cmdr::TreeCompleter::build(&read.data.cmdtree)),
                    Box::new(modules::ModulesCompleter::build(
                        &read.data.cmdtree,
                        read.data.file_map(),
                    )),
                    // Box::new(code::CodeCompleter::build(&read.data)),
                ],
            };

            read.set_completion(combined);

            read.draw_prompt2();

            read.read_line(&mut term);

            match read.read2() {
                ReadResult::Read(repl) => read = repl,
                ReadResult::Eval(repl) => {
                    let result = repl.eval(app_data);

                    match result.signal {
                        Signal::None => (),
                        Signal::Exit => break,
                    }

                    read = result.repl.print();
                }
            }
        }
    }

    fn read_line(&mut self, term: &mut mortal::Terminal) {
        use mortal::{Event, Key};

        loop {
            match term
                .read_event(None)
                .unwrap_or(None)
                .unwrap_or(Event::NoEvent)
            {
                Event::Key(k) => match k {
                    Key::Char(ch) => self.input_ch(ch),
                    Key::Enter => break, // new line found!
                    _ => (),
                },
                _ => (),
            }
        }
    }
}

fn output_repl(rx: output::Receiver) -> std::io::Result<()> {
    let term = mortal::Terminal::new()?;

    let mut last_total = 1;

    // Map of how many lines a line index prints to.
    let mut line_lens = vec![1];

    let mut lock = term.lock_write().unwrap();

    for msg in rx.iter() {
        let size = lock.size()?;

        // add necessary new lines. Indices increment by one.
        {
            for _ in 0..(msg.total.saturating_sub(last_total)) {
                line_lens.push(1);
                writeln!(lock, "")?;
            }

            last_total = msg.total;

            debug_assert_eq!(line_lens.len(), last_total);
        }

        // move to, and clear line
        {
            let diff = line_lens[msg.index..]
                .iter()
                .sum::<usize>()
                .saturating_sub(1);
            lock.move_up(diff)?;
            lock.move_to_first_column()?;
            lock.clear_to_line_end()?;
        }

        // write contents, might spill over into multiple lines
        {
            let lines_count = {
                let chars = msg.line.chars().count();

                if chars == 0 {
                    1
                } else {
                    let r = chars % size.columns;
                    if r == 0 {
                        chars / size.columns
                    } else {
                        chars / size.columns + 1
                    }
                }
            };

            write!(lock, "{}", msg.line)?;

            line_lens.get_mut(msg.index).map(|x| *x = lines_count);
        }

        // move cursor to last line
        {
            let diff = line_lens[msg.index..]
                .iter()
                .sum::<usize>()
                .saturating_sub(1);
            lock.move_down(diff)?;
            lock.move_to_first_column()?;
        }

        lock.flush()?;
    }

    Ok(())
}
