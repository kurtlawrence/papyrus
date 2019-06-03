mod any_state;
mod printer;
mod read;
mod write;

pub use printer::Printer;

use crossbeam_channel as channel;

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

    tx: Option<channel::Sender<LineChange>>,
}

#[derive(Debug)]
pub struct LineChange {
    pub line_index: usize,
    pub line: String,
}

#[derive(Debug)]
pub struct Read {
    /// Byte position that starts the input buffer.
    input_start: usize,
}

#[derive(Debug)]
pub struct Write;

enum Char {
    Ch(char),
    Tab,
    Return,
    Backspace,
}

// Altering output buffer functions.
impl<S> Output<S> {
    /// Push a character onto end of buffer.
    ///
    /// The character is interrogated and the following behaviours are followed.
    ///
    /// | char                     | action                              |
    /// | ------------------------ | ----------------------------------- |
    /// | Carriage Return (`'\r'`) | ignored                             |
    /// | New Line (`'\n'`)        | append, add line                    |
    /// | Backspace (`'\x08'`)     | pop, _only if not at start of line_ |
    /// | Tab (`'\t'`)             | append _four_ spaces                |
    /// | Other                    | append                              |
    ///
    /// # Line Changes
    /// Sends a line change message if a new line is reached. Otherwise no line change signal is sent.
    fn push_ch(&mut self, ch: char) {
        match ch {
            '\r' => (), // carrige returns are ignored
            '\n' => {
                self.lines_pos.push(self.buf.len());
                self.buf.push('\n');

                // send line change signal of this line
                let idx = self.lines_len().saturating_sub(2);
                self.send_line_chg(
                    idx,
                    self.line(idx)
                        .map(|x| x.to_string())
                        .unwrap_or(String::new()),
                );
            }
            '\x08' => {
                self.pop();
            }
            '\t' => self.buf.push_str("    "),
            x => self.buf.push(x),
        }
    }

    /// Iterates characters and pushes each one using `push_ch`.
    ///
    /// # Line Changes
    /// Line change signal is sent for _all_ lines changed.
    fn push_str(&mut self, string: &str) {
        string.chars().for_each(|ch| self.push_ch(ch));

        // send line change signal of last line -- previous ones are handled in push_ch
        let idx = self.lines_len().saturating_sub(1);
        self.send_line_chg(
            idx,
            self.line(idx)
                .map(|x| x.to_string())
                .unwrap_or(String::new()),
        );
    }

    /// Only pops input if not at start of new line.
    ///
    /// # Line Changes
    /// No line change signal is sent.
    fn pop(&mut self) -> Option<char> {
        if !self.at_line_start() {
            self.buf.pop()
        } else {
            None
        }
    }

    fn at_line_start(&self) -> bool {
        self.buf.len() == 0
            || self
                .lines_pos
                .last()
                .map(|&x| x == self.buf.len() - 1)
                .unwrap_or(false)
    }
}

// Message sending functions.
impl<S> Output<S> {
    fn send_line_chg(&mut self, line_index: usize, line: String) {
        if let Some(tx) = self.tx.as_ref() {
            match tx.try_send(LineChange { line_index, line }) {
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
    fn at_line_start() {
        let mut o = Output::default();

        assert_eq!(o.at_line_start(), true);

        o.push_str("Hello");
        assert_eq!(o.at_line_start(), false);

        o.push_str("\n");
        assert_eq!(o.at_line_start(), true);

        o.push_str("world");
        assert_eq!(o.at_line_start(), false);
    }

    #[test]
    fn lines_data_structure() {
        // check that the vec of positions is suitable

        let mut o = Output::new();

        assert_eq!(o.lines_pos, vec![]);
        assert_eq!(o.line(0), Some(""));

        o.push_str("Hello");

        assert_eq!(o.lines_pos, vec![]);
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

        o.push_str("\t");
        assert_eq!(o.buffer(), "Hello, World!\n    ");

        o.push_str("\x08\x08\x08\x08\x08\x08\x08\x08\x08\x08");
        assert_eq!(o.buffer(), "Hello, World!\n");

        let mut o = Output::new();

        o.push_str("Hello");
        o.push_str("\x08\x08\x08\x08\x08\x08\x08\x08\x08\x08");
        assert_eq!(o.buffer(), "");
    }

    #[test]
    fn pop() {
        let mut o = Output::new();

        assert_eq!(o.pop(), None);

        o.push_str("H");
        assert_eq!(o.pop(), Some('H'));
        assert_eq!(o.pop(), None);

        o.push_str("\nHe");
        assert_eq!(o.pop(), Some('e'));
        assert_eq!(o.pop(), Some('H'));
        assert_eq!(o.pop(), None);
    }
}
