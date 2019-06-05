use super::*;
use mortal::{Event, Key};

#[cfg(feature = "runnable")]
#[cfg(feature = "racer-completion")]
impl<Term: 'static + Terminal, Data> Repl<Read, Term, Data> {
    /// Run the REPL interactively. Consumes the REPL in the process and will block this thread until exited.
    ///
    /// # Panics
    /// - Failure to initialise `InputReader`.
    pub fn run(self, app_data: &mut Data) {
        let mut term = mortal::Terminal::new().unwrap();

        output_ver(self.terminal.terminal.as_ref());

        let mut read = self;

        // output to stdout
        let rx = read.output_listen();
        std::thread::spawn(move || output_repl(rx));

        loop {
            let combined = CombinedCompleter {
                completers: vec![
                    Box::new(cmdr::TreeCompleter::build(&read.data.cmdtree)),
                    Box::new(modules::ModulesCompleter::build(
                        &read.data.cmdtree,
                        &read.data.file_map,
                    )),
                    Box::new(code::CodeCompleter::build(&read.data)),
                ],
            };

            read.set_completion(combined);

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
    use std::io::Write;

    let term = mortal::Terminal::new()?;

    let mut last_total = 1;

    for msg in rx.iter() {
        for _ in 0..(msg.total.saturating_sub(last_total)) {
            writeln!(term, "")?;
        }

        last_total = msg.total;

        let diff = msg.total.saturating_sub(msg.index).saturating_sub(1);

        term.move_up(diff)?;
        term.move_to_first_column()?;
        term.clear_to_line_end()?;

        write!(term, "{}", msg.line)?;

        term.move_down(diff)?;
    }

    Ok(())
}

#[cfg(feature = "runnable")]
fn output_ver<T: Terminal>(term: &T) {
    cratesiover::output_to_writer("papyrus", env!("CARGO_PKG_VERSION"), &mut Writer(term)).unwrap();
}
