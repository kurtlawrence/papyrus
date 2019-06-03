use super::*;

impl<Term: 'static + Terminal, Data> Repl<Read, Term, Data> {
    /// Run the REPL interactively. Consumes the REPL in the process and will block this thread until exited.
    ///
    /// # Panics
    /// - Failure to initialise `InputReader`.
    #[cfg(feature = "runnable")]
    #[cfg(feature = "racer-completion")]
    pub fn run(self, app_data: &mut Data) {
        output_ver(self.terminal.terminal.as_ref());

        let mut read = self;

        // start up writing received lines to terminal
        let rx = read.output_listen();
        std::thread::spawn(move || {
            for msg in rx.iter() {
                println!("{:#?}", msg);
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
