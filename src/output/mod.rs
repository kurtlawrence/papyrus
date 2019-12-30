//! Reading and writing output.
//!
//! The [`Repl`] provides mechanisms to handle the output that is produced when operating. An
//! [`Output`] instance is maintained in the `Repl`. This output is vector of lines of output. There
//! are also mechanisms on the `Repl` to listen to changes on the `Output`, such that changes can be
//! synchronised without needing to diff the output. This is useful in longer running operations where
//! displaying the output progressively is required.
//!
//! # Examples
//! ## Stdio
//!
//! Stdio could be construed as the more simple case, but actually entails some more complexity due to
//! the dual nature of input and output in the terminal, whereas if input is separated from the output
//! rendering the [separated rendering](#separated-rendering) example can be implemented.
//!
//! This tutorial works through the example at
//! [`papyrus/examples/output-stdio.rs`](https://github.com/kurtlawrence/papyrus/blob/master/papyrus/examples/output-stdio.rs).
//!
//! First start an empty binary project with two extra dependencies:
//! [term_cursor](https://crates.io/crates/term_cursor) and
//! [term_size](https://crates.io/crates/term_size).
//!
//! ```rust,ignore
//! #[macro_use]
//! extern crate papyrus;
//! extern crate term_cursor;
//! extern crate term_size;
//!
//! use papyrus::output::{OutputChange, Receiver};
//! use papyrus::repl::{EvalResult, ReadResult, Signal};
//! use std::io::{stdin, stdout, StdoutLock, Write};
//! ```
//!
//! Before defining the main loop, lets define some functions that will be used. The first one is one
//! which erases the current console line. It does this by moving to the first column in the terminal,
//! writing a line of spaces (' '), and then moving again to the first column. This is where the
//! dependencies are required.
//!
//! ```rust,ignore
//! /// Erases the current console line by moving to first column,
//! /// writing a row of spaces ' ', then setting cursor back
//! /// to first column.
//! fn erase_console_line(stdout: &mut StdoutLock) {
//!     use term_cursor as cursor;
//!     let lineidx = cursor::get_pos().map(|x| x.1).unwrap_or_default();
//!
//!     let width = term_size::dimensions().map(|(w, _)| w).unwrap_or_default();
//!
//!     cursor::set_pos(0, lineidx).unwrap();
//!     (0..width).for_each(|_| write!(stdout, " ").unwrap());
//!     cursor::set_pos(0, lineidx).unwrap();
//! }
//! ```
//!
//! Next we define a simple fuction to read a line from stdin and return a string.
//!
//! ```rust,ignore
//! /// Waits for a line input from stdin.
//! fn read_line() -> String {
//!     let mut buf = String::new();
//!     stdin().read_line(&mut buf).unwrap();
//!     buf
//! }
//! ```
//!
//! And finally define a function that handles the line changes. This function does its work on a
//! separate thread to run asynchronously while the repl is evaluating. It receives each line change
//! and writes it to stdout, erasing a current line, or writing a new line if required.
//!
//! ```rust,ignore
//! /// This handles the writing of output changes to stdout.
//! fn write_output_to_stdout(rx: Receiver) -> std::thread::JoinHandle<()> {
//!     std::thread::spawn(move || {
//!         let mut stdout = std::io::stdout();
//!
//!         for chg in rx.iter() {
//!             match chg {
//!                 OutputChange::CurrentLine(line) => {
//!                     let mut lock = stdout.lock();
//!                     erase_console_line(&mut lock);
//!                     write!(&mut lock, "{}", line).unwrap();
//!                     lock.flush().unwrap();
//!                 }
//!                 OutputChange::NewLine => writeln!(&mut stdout, "").unwrap(),
//!             }
//!         }
//!     })
//! }
//! ```
//!
//! These functions can be used in a main function that handles the repl states.
//!
//! ```rust,ignore
//! fn main() {
//!     // build the repl
//!     let repl = repl!();
//!
//!     // alias the state in the variable name.
//!     let mut read = repl;
//!
//!     loop {
//!         // write the prompt, erase line first.
//!         {
//!             let stdout = stdout();
//!             let mut lock = stdout.lock();
//!             erase_console_line(&mut lock);
//!             write!(&mut lock, "{}", read.prompt()).unwrap();
//!             lock.flush().unwrap();
//!         }
//!
//!         // read line from stdin
//!         let line_input = read_line();
//!
//!         // set this as the repl input
//!         read.line_input(&line_input);
//!
//!         // handle the input and get a result from it
//!         let read_res = read.read();
//!
//!         match read_res {
//!             ReadResult::Read(repl) => {
//!                 // The repl is still in a read state, continue reading
//!                 read = repl;
//!             }
//!             ReadResult::Eval(mut eval) => {
//!                 // The repl is ready for evaluating
//!
//!                 // as we want to update as input comes in,
//!                 // we need to listen to output changes
//!                 let rx = eval.output_listen();
//!
//!                 // start the output thread
//!                 let output_thread_jh = write_output_to_stdout(rx);
//!
//!                 // evaluate using a unit value for data
//!                 let EvalResult { repl, signal } = eval.eval(&mut ());
//!
//!                 // handle the signal, other values are elided but would be
//!                 // handled in a more complete implementation
//!                 match signal {
//!                     Signal::Exit => break,
//!                     _ => (),
//!                 }
//!
//!                 let (mut repl, _) = repl.print();
//!
//!                 // we have printed everything it is time to close the channel
//!                 // it is worth testing what happens if you don't, and it should
//!                 // highlight the reason for requiring the listening channels.
//!                 repl.close_channel();
//!
//!                 // we wait for the output thread to finish to let it write out
//!                 // the remaining lines
//!                 output_thread_jh.join().unwrap();
//!
//!                 read = repl;
//!             }
//!         }
//!     }
//! }
//! ```
//!
//!
//! ## Separated Rendering
//!
//! To reduce the complexity of rendering, the output listener and listen to _all_ changes, which
//! includes any input changes, it will get updated on calls to `Repl.line_input()`. This example shows
//! a naive implementation which writes to a file as it goes.
//!
//! This tutorial works through the example at
//! [`papyrus/examples/output-file.rs`](https://github.com/kurtlawrence/papyrus/blob/master/papyrus/examples/output-file.rs).
//!
//! First start an empty binary project:
//!
//! ```rust,ignore
//! #[macro_use]
//! extern crate papyrus;
//!
//! use papyrus::output::{OutputChange, Receiver};
//! use papyrus::repl::{EvalResult, ReadResult, Signal};
//! use std::io::{stdin, StdoutLock, Write};
//! ```
//!
//! As before there is a `read_line` function, and we alter the output function somewhat.
//!
//! ```rust,ignore
//! /// Waits for a line input from stdin.
//! fn read_line() -> String {
//!     let mut buf = String::new();
//!     stdin().read_line(&mut buf).unwrap();
//!     buf
//! }
//!
//! /// Write output to a file as you go. Not a very efficient implementation.
//! fn write_outoput_to_file(rx: Receiver) -> std::thread::JoinHandle<()> {
//!     std::thread::spawn(move || {
//!         let mut output = String::new();
//!         let mut pos = 0;
//!
//!         for chg in rx.iter() {
//!             match chg {
//!                 OutputChange::CurrentLine(line) => {
//!                     output.truncate(pos);
//!                     output.push_str(&line);
//!                     std::fs::write("repl-output.txt", &output).unwrap();
//!                 }
//!                 OutputChange::NewLine => {
//!                     output.push('\n');
//!                     pos = output.len();
//!                 }
//!             }
//!         }
//!     })
//! }
//! ```
//!
//! These functions can be used in a main function that handles the repl states.
//!
//! ```rust,ignore
//! fn main() {
//!     // build the repl
//!     let repl = repl!();
//!
//!     // alias the state in the variable name.
//!     let mut read = repl;
//!
//!     // as we want to update as input comes in,
//!     // we need to listen to output changes
//!     let rx = read.output_listen();
//!
//!     // start the output thread
//!     write_outoput_to_file(rx);
//!
//!     loop {
//!         // read line from stdin
//!         let line_input = read_line();
//!
//!         // set this as the repl input
//!         read.line_input(&line_input);
//!
//!         // handle the input and get a result from it
//!         let read_res = read.read();
//!
//!         read = match read_res {
//!             ReadResult::Read(repl) => {
//!                 // The repl is still in a read state, continue reading
//!                 repl
//!             }
//!             ReadResult::Eval(eval) => {
//!                 // The repl is ready for evaluating
//!
//!                 // evaluate using a unit value for data
//!                 let EvalResult { repl, signal } = eval.eval(&mut ());
//!
//!                 // handle the signal, other values are elided but would be
//!                 // handled in a more complete implementation
//!                 match signal {
//!                     Signal::Exit => break,
//!                     _ => (),
//!                 }
//!
//!                 let (repl, _) = repl.print();
//!                 repl
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! [`Output`]: crate::output::Output
//! [`Repl`]: crate::repl::Repl
mod any_state;
mod read;
mod write;

use crossbeam_channel as channel;

/// Line change receiving end.
pub type Receiver = channel::Receiver<OutputChange>;

/// Represents a buffered output from the repl.
///
/// Output can be either the `Read` or `Write` states,
/// and can be listened to for line change events.
#[derive(Debug)]
pub struct Output<S> {
    state: S,

    /// **BE CAREFUL** when altering buffer to ensure lines state and other discriminants
    /// are upheld. Use the inner functions rather than directly altering buffer.
    buf: String,

    /// _Exclusive_ ending index of each line.
    ///
    /// First line always starts `[0..line_pos[0])` (can be empty).
    /// Every `nth` line after that is `[line_pos[n-1] + 1..line_pos[n])`.
    lines_pos: Vec<usize>,

    tx: Option<channel::Sender<OutputChange>>,
}

/// Line change event.
#[derive(Debug, PartialEq)]
pub enum OutputChange {
    /// A change was made on the current line.
    CurrentLine(String),
    /// Output is on a new line now.
    NewLine,
}

/// Only read functions available.
#[derive(Debug)]
pub struct Read {
    /// Input buffer. This contains _all_ input, which may include previous lines.
    buf: String,

    start: usize,

    prompt_start: usize,
    prompt_end: usize,

    lines_idx: usize,
}

/// Only write functions available.
#[derive(Debug)]
pub struct Write;

/// Altering output buffer functions.
impl<S> Output<S> {
    /// Push a character onto end of buffer. Returns if line change event
    /// was sent.
    ///
    /// The character is interrogated and the following behaviours are followed.
    ///
    /// | char                     | action                              |
    /// | ------------------------ | ----------------------------------- |
    /// | Carriage Return (`'\r'`) | ignored                             |
    /// | New Line (`'\n'`)        | append, add line                    |
    /// | Other                    | append                              |
    ///
    /// # Line Changes
    /// Sends a line change message if a new line is reached (`'\n'`).
    /// Otherwise no line change signal is sent.
    fn push_ch(&mut self, ch: char) -> bool {
        match ch {
            '\r' => false, // carrige returns are ignored
            '\n' => {
                // send line change signal of this line
                self.send_line_chg();
                self.send_newline();

                self.lines_pos.push(self.buf.len());

                self.buf.push('\n');

                true
            }
            x => {
                self.buf.push(x);
                false
            }
        }
    }

    /// Iterates characters and pushes each one using `push_ch`.
    ///
    /// # Line Changes
    /// Line change signal is sent for _all_ lines changed.
    fn push_str(&mut self, string: &str) {
        string.chars().for_each(|ch| {
            self.push_ch(ch);
        });

        // send line change signal of last line -- previous ones are handled in push_ch
        self.send_line_chg();
    }
}

/// Message sending functions.
impl<S> Output<S> {
    /// Always sends the last line content.
    fn send_line_chg(&mut self) {
        if let Some(tx) = self.tx.as_ref() {
            let line = self
                .line(self.lines_len().saturating_sub(1))
                .unwrap_or("")
                .to_string();

            match tx.try_send(OutputChange::CurrentLine(line)) {
                Ok(_) => (),
                Err(_) => self.tx = None, // receiver disconnected, stop sending msgs
            }
        }
    }

    fn send_newline(&mut self) {
        if let Some(tx) = self.tx.as_ref() {
            match tx.try_send(OutputChange::NewLine) {
                Ok(_) => (),
                Err(_) => self.tx = None, // receiver disconnected, stop sending msgs
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_data_structure() {
        // check that the vec of positions is suitable

        let mut o = Output::new();

        assert_eq!(o.lines_pos, Vec::<usize>::new());
        assert_eq!(o.line(0), Some(""));

        o.push_str("Hello");

        assert_eq!(o.lines_pos, Vec::<usize>::new());
        assert_eq!(o.line(0), Some("Hello"));

        o.push_str("\nworld");

        assert_eq!(o.lines_pos, vec![5]);
        assert_eq!(o.line(0), Some("Hello"));
        assert_eq!(o.line(1), Some("world"));
        assert_eq!(o.line(2), None);

        o.push_str("\n");

        assert_eq!(o.lines_pos, vec![5, 11]);
        assert_eq!(o.line(0), Some("Hello"));
        assert_eq!(o.line(1), Some("world"));
        assert_eq!(o.line(2), Some(""));
        assert_eq!(o.line(3), None);
    }

    #[test]
    fn push_ch() {
        let mut o = Output::new();

        o.push_str("Hello, World!");
        assert_eq!(o.buffer(), "Hello, World!");

        o.push_str("\r\n");
        assert_eq!(o.buffer(), "Hello, World!\n");
    }

    #[test]
    fn writeln_as_one_line_chg_trigger() {
        // It was found that using write! or writeln! seems to make multiple calls to
        // line changes. This behaviour is inconsistent, especially if using the writeln!
        // macro, where the expectation is just a single new line...

        let mut output = Output::new().to_write();

        let rx = output.listen();

        output.push_str(&format!("{}, {}!\n", "Hello", "world"));
        output.push_str("Testing\nmultiple\nnew\nlines");

        output.close();

        use OutputChange as oc;

        let msgs = rx.iter().collect::<Vec<_>>();

        assert_eq!(
            &msgs,
            &[
                oc::CurrentLine("Hello, world!".to_owned()),
                oc::NewLine,
                oc::CurrentLine(String::new()),
                oc::CurrentLine(String::from("Testing")),
                oc::NewLine,
                oc::CurrentLine(String::from("multiple")),
                oc::NewLine,
                oc::CurrentLine(String::from("new")),
                oc::NewLine,
                oc::CurrentLine(String::from("lines")),
            ]
        );
    }

    #[test]
    fn line_listening_consistenty() {
        // check that rebuilding a buffer using line change events matches

        let mut output = Output::new().to_write();

        let rx = output.listen();

        output.push_str("Hello,");
        output.push_str(" world!");
        output.push_str("\n");
        output.push_str("A full line\n");

        output.close();

        let mut lines = vec![String::new()];

        for msg in rx.iter() {
            match msg {
                OutputChange::CurrentLine(s) => {
                    lines.last_mut().map(|x| *x = s);
                }
                OutputChange::NewLine => lines.push(String::new()),
            }
        }

        let buffer = lines.join("\n");

        assert_eq!(buffer, output.buffer());
    }
}
