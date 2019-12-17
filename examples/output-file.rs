#[macro_use]
extern crate papyrus;

// This example shows how to use the Output listeners
// to build a repl that works with stdin and writes to a file.

use papyrus::output::{OutputChange, Receiver};
use papyrus::repl::{EvalResult, ReadResult, Signal};
use std::io::stdin;

fn main() {
    // build the repl
    let repl = repl!();

    // alias the state in the variable name.
    let mut read = repl;

    // as we want to update as input comes in,
    // we need to listen to output changes
    let rx = read.output_listen();

    // start the output thread
    write_outoput_to_file(rx);

    loop {
        // read line from stdin
        let line_input = read_line();

        // set this as the repl input
        read.line_input(&line_input);

        // handle the input and get a result from it
        let read_res = read.read();

        read = match read_res {
            ReadResult::Read(repl) => {
                // The repl is still in a read state, continue reading
                repl
            }
            ReadResult::Eval(eval) => {
                // The repl is ready for evaluating

                // evaluate using a unit value for data
                let EvalResult { repl, signal } = eval.eval(&mut ());

                // handle the signal, other values are elided but would be
                // handled in a more complete implementation
                match signal {
                    Signal::Exit => break,
                    _ => (),
                }

                let (repl, _) = repl.print();
                repl
            }
        }
    }
}

/// Waits for a line input from stdin.
fn read_line() -> String {
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    buf
}

/// Write output to a file as you go. Not a very efficient implementation.
fn write_outoput_to_file(rx: Receiver) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut output = String::new();
        let mut pos = 0;

        for chg in rx.iter() {
            match chg {
                OutputChange::CurrentLine(line) => {
                    output.truncate(pos);
                    output.push_str(&line);
                    std::fs::write("repl-output.txt", &output).unwrap();
                }
                OutputChange::NewLine => {
                    output.push('\n');
                    pos = output.len();
                }
            }
        }
    })
}
