use crate::prelude::*;
use linefeed::Terminal;
use crate::complete::*;
use repl::{Read, ReadResult, Signal};
use crate::output;

// #[cfg(feature = "racer-completion")]
impl<Term: 'static + Terminal, Data> Repl<Read, Term, Data> {
    /// Run the REPL interactively. Consumes the REPL in the process and will block this thread until exited.
    ///
    /// # Panics
    /// - Failure to initialise `InputReader`.
    pub fn run(self, app_data: &mut Data) {
        let mut term = mortal::Terminal::new().unwrap();

		cratesiover::output_to_writer("papyrus", env!("CARGO_PKG_VERSION"), &mut std::io::stdout()).unwrap();

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

    let mut lock = term.lock_write().unwrap();

    for msg in rx.iter() {
        for _ in 0..(msg.total.saturating_sub(last_total)) {
            writeln!(lock, "")?;
        }

        last_total = msg.total;

        let diff = msg.total.saturating_sub(msg.index).saturating_sub(1);

        lock.move_up(diff)?;
        lock.move_to_first_column()?;
        lock.clear_to_line_end()?;

        write!(lock, "{}", msg.line)?;

        lock.move_down(diff)?;

        lock.flush()?;
    }

    Ok(())
}

