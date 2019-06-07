use crate::complete::*;
use crate::output;
use crate::prelude::*;
use linefeed::Terminal;
use linefeed::{DefaultTerminal, Interface};
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

        let mut term = Interface::new("papyrus").unwrap();

        let mut read = self;

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

            let input = match term.read_line().unwrap() {
                linefeed::ReadResult::Input(s) => s,
                _ => String::new(),
            };

            read.line_input(&input);

            // read.read_line(&mut term);

            match read.read2() {
                ReadResult::Read(repl) => read = repl,
                ReadResult::Eval(mut repl) => {
                    // output to stdout
                    let rx = repl.output_listen();
                    std::thread::spawn(move || output_repl(rx).unwrap());

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
}

fn output_repl(rx: output::Receiver) -> std::io::Result<()> {
    use term_cursor as cursor;
    use term_size as size;

    let term = DefaultTerminal::new()?;

    let mut pos = cursor::get_pos().unwrap_or((0, 0));

    for msg in rx.iter() {
        match msg {
            output::OutputChange::CurrentLine(line) => {
                let mut o = term.lock_write();
                let p = cursor::get_pos().unwrap_or((0, 0));
                let diff = (p.1 as usize).saturating_sub(pos.1 as usize);
                o.move_up(diff);
                o.move_to_first_column();
                o.clear_to_screen_end();
                o.write(&line);
                o.flush();
            }
            output::OutputChange::NewLine(line) => {
                let mut o = term.lock_write();
                o.write("\n");
                pos = cursor::get_pos().unwrap_or((0, 0));
                o.write(&line);
                o.flush()?;
            }
        }
    }

    // let term = mortal::Terminal::new()?;

    // let mut last_total = 1;

    // // Map of how many lines a line index prints to.
    // let mut line_lens = vec![1];

    // let mut lock = term.lock_write().unwrap();

    // for msg in rx.iter() {
    //     let size = lock.size()?;

    //     // add necessary new lines. Indices increment by one.
    //     {
    //         for _ in 0..(msg.total.saturating_sub(last_total)) {
    //             line_lens.push(1);
    //             writeln!(lock, "")?;
    //         }

    //         last_total = msg.total;

    //         debug_assert_eq!(line_lens.len(), last_total);
    //     }

    //     // move to, and clear line
    //     let cols = {
    //         let diff = line_lens[msg.index..]
    //             .iter()
    //             .sum::<usize>()
    //             .saturating_sub(1);
    //         lock.move_up(diff)?;
    //         let cols = mv_to_first_col(&mut lock);
    //         lock.clear_to_line_end()?;
    //         cols
    //     };

    //     // write contents, might spill over into multiple lines
    //     {
    //         let lines_count = {
    //             let chars = cansi::categorise_text(&msg.line)
    //                 .iter()
    //                 .map(|c| c.text.chars().count())
    //                 .sum::<usize>();

    //             if chars == 0 {
    //                 1
    //             } else {
    //                 let r = chars % size.columns;

    //                 if r == 0 {
    //                     chars / size.columns
    //                 } else {
    //                     chars / size.columns + 1
    //                 }
    //             }
    //         };

    //         write!(lock, "{}", msg.line)?;

    //         line_lens.get_mut(msg.index).map(|x| *x = lines_count);
    //     }

    //     // move cursor to last line
    //     {
    //         let diff = line_lens[msg.index..]
    //             .iter()
    //             .sum::<usize>()
    //             .saturating_sub(1);

    //         if msg.index != last_total - 1 {
    //             lock.move_down(diff)?;
    //             lock.move_to_first_column()?;
    //             lock.move_right(cols)?;
    //         }
    //     }

    //     lock.flush()?;
    // }

    Ok(())
}

fn mv_to_first_col(lock: &mut mortal::TerminalWriteGuard) -> usize {
    let mut cols = 0;

    while let Ok(_) = lock.move_left(1) {
        cols += 1;
    }

    cols
}
