#[macro_use]
extern crate papyrus;

// This example shows how to use the Output listeners
// to build a repl that works with stdin and stdout.

use papyrus::output::{OutputChange, Receiver};
use papyrus::repl::{EvalResult, ReadResult, Signal};
use std::io::{stdin, stdout, StdoutLock, Write};

fn main() {
    // build the repl
    let repl = repl!();

    // alias the state in the variable name.
    let mut read = repl;

    loop {
        // write the prompt, erase line first.
        {
            let stdout = stdout();
            let mut lock = stdout.lock();
            erase_console_line(&mut lock);
            write!(&mut lock, "{}", read.prompt(true)).unwrap();
            lock.flush().unwrap();
        }

        // read line from stdin
        let line_input = read_line();

        // set this as the repl input
        read.line_input(&line_input);

        // handle the input and get a result from it
        let read_res = read.read();

        match read_res {
            ReadResult::Read(repl) => {
                // The repl is still in a read state, continue reading
                read = repl;
            }
            ReadResult::Eval(mut eval) => {
                // The repl is ready for evaluating

                // as we want to update as input comes in,
                // we need to listen to output changes
                let rx = eval.output_listen();

                // start the output thread
                let output_thread_jh = write_output_to_stdout(rx);

                // evaluate using a unit value for data
                let EvalResult { repl, signal } = eval.eval(&mut ());

                // handle the signal, other values are elided but would be
                // handled in a more complete implementation
                match signal {
                    Signal::Exit => break,
                    _ => (),
                }

                let (mut repl, _) = repl.print();

                // we have printed everything it is time to close the channel
                // it is worth testing what happens if you don't, and it should
                // highlight the reason for requiring the listening channels.
                repl.close_channel();

                // we wait for the output thread to finish to let it write out
                // the remaining lines
                output_thread_jh.join().unwrap();

                read = repl;
            }
        }
    }
}

fn erase_console_line(stdout: &mut StdoutLock) {
    use term_cursor as cursor;
    let lineidx = cursor::get_pos().map(|x| x.1).unwrap_or_default();

    let width = term_size::dimensions().map(|(w, _)| w).unwrap_or_default();

    cursor::set_pos(0, lineidx).unwrap();
    (0..width).for_each(|_| write!(stdout, " ").unwrap());
    cursor::set_pos(0, lineidx).unwrap();
}

/// Waits for a line input from stdin.
fn read_line() -> String {
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    buf
}

/// This handles the writing of output changes to stdout.
/// Notice the use of term_cursor;
fn write_output_to_stdout(rx: Receiver) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut stdout = std::io::stdout();

        for chg in rx.iter() {
            match chg {
                OutputChange::CurrentLine(line) => {
                    let mut lock = stdout.lock();
                    erase_console_line(&mut lock);
                    write!(&mut lock, "{}", line).unwrap();
                    lock.flush().unwrap();
                }
                OutputChange::NewLine => writeln!(&mut stdout, "").unwrap(),
            }
        }
    })
}
