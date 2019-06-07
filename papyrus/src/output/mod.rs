mod any_state;
mod read;
mod write;

use crossbeam_channel as channel;

/// Line change receiving end.
pub type Receiver = channel::Receiver<OutputChange>;

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
#[derive(Debug)]
pub enum OutputChange {
    CurrentLine(String),
    NewLine(String),
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
                self.send_line_chg(true);

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
        self.send_line_chg(false);
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

/// Message sending functions.
impl<S> Output<S> {
    /// Always sends the last line, but whether it is flagged as a new line or not is up to caller.
    fn send_line_chg(&mut self, new_line: bool) {
        if let Some(tx) = self.tx.as_ref() {
            let line = self
                .line(self.lines_len().saturating_sub(1))
                .unwrap_or("")
                .to_string();

            let chg = if new_line {
                OutputChange::NewLine(line)
            } else {
                OutputChange::CurrentLine(line)
            };

            match tx.try_send(chg) {
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
}
