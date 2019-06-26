use super::*;
use std::io;

/// Write state functions.
impl Output<Write> {
    /// Finished write state. Move to read state.
    ///
    /// The input buffer is initialised as empty.
    pub fn to_read(self) -> Output<Read> {
        let Output {
            buf, lines_pos, tx, ..
        } = self;

        let state = Read {
            buf: String::new(),
            start: 0,
            prompt_start: buf.len(),
            prompt_end: buf.len(),
            lines_idx: lines_pos.len(),
        };

        Output {
            state,
            buf,
            lines_pos,
            tx,
        }
    }

    /// Writes the string contents to the end of the buffer.
    ///
    /// # Line Changes
    /// Triggers a line change event for _all_ changed lines
    /// (say if there are new lines in the string).
    pub fn write_str(&mut self, contents: &str) {
        self.push_str(contents);
    }

    pub fn write_line(&mut self, contents: &str) {
        for ch in contents.chars() {
            self.push_ch(ch);
        }
        self.push_ch('\n');
    }

    /// Erase the last line in the buffer. This does not actually _remove_
    /// the line, but removes all its contents.
    ///
    /// # Line Changes
    /// Triggers a line change event.
    ///
    /// # Examples
    /// ```rust
    /// # use papyrus::output::Output;
    ///
    /// let mut o = Output::new().to_write();
    ///
    /// o.write_str("Hello\nworld");
    /// o.erase_last_line();
    /// assert_eq!(o.buffer(), "Hello\n");
    ///
    /// o.erase_last_line(); // keeps line.
    /// assert_eq!(o.buffer(), "Hello\n");
    /// ```
    pub fn erase_last_line(&mut self) {
        self.buf
            .truncate(self.lines_pos.last().map(|x| x + 1).unwrap_or(0));

        self.send_line_chg();
    }
}

impl io::Write for Output<Write> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        dbg!(&s);
        self.push_str(&s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erase_last_line() {
        let mut o = Output::new().to_write();

        o.write_str("Hello\nworld");
        o.erase_last_line();
        assert_eq!(o.buffer(), "Hello\n");
        o.erase_last_line();
        assert_eq!(o.buffer(), "Hello\n");

        let mut o = Output::new().to_write();

        o.write_str("Hello");
        o.erase_last_line();
        assert_eq!(o.buffer(), "");
    }

    #[test]
    fn writing_line() {
        let mut o = Output::new().to_write();

        let rx = o.listen();

        o.write_line("Hello, world!");

        o.close();

        let msgs = rx.iter().collect::<Vec<_>>();

        assert_eq!(o.buffer(), "Hello, world!\n");

        assert_eq!(
            &msgs,
            &[
                OutputChange::CurrentLine("Hello, world!".to_owned()),
                OutputChange::NewLine
            ]
        );
    }
}
