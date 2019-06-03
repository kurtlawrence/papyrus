use super::*;

impl<Term: 'static + Terminal, Data> Repl<Read, Term, Data> {
    /// Run the REPL interactively. Consumes the REPL in the process and will block this thread until exited.
    ///
    /// # Panics
    /// - Failure to initialise `InputReader`.
    #[cfg(feature = "runnable")]
    #[cfg(feature = "racer-completion")]
    pub fn run(self, app_data: &mut Data) {
        use std::io::Write;
        use term_cursor as cursor;

        output_ver(self.terminal.terminal.as_ref());

        let mut read = self;

        // start up writing received lines to terminal

        // effectively this is output line idx of 0.
        let start_y = cursor::get_pos().map(|x| x.1).unwrap_or(0);

        let rx = read.output_listen();

        std::thread::spawn(move || {
            for msg in rx.iter() {
                let (w, _) = term_size::dimensions().unwrap_or((0, 0));
                let (x, y) = cursor::get_pos().unwrap_or((0, 0));

                let line: i32 = start_y + msg.line_index as i32;

                // first erase contents of line
                cursor::set_pos(0, line).ok();
                (0..w).into_iter().for_each(|_| print!(" "));

                // then write line
                cursor::set_pos(0, line).ok();
                print!("{}", msg.line);

                // reset cursor positon
                cursor::set_pos(x, y).ok();

                // flush changes
                std::io::stdout().flush().ok();
            }
        });

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

            let result = read.read().eval(app_data);

            match result.signal {
                Signal::None => (),
                Signal::Exit => break,
            }

            read = result.repl.print();
        }
    }
}

#[cfg(feature = "runnable")]
fn output_ver<T: Terminal>(term: &T) {
    cratesiover::output_to_writer("papyrus", env!("CARGO_PKG_VERSION"), &mut Writer(term)).unwrap();
}
