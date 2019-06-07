use super::*;
use std::io;

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

        self.send_line_chg(self.lines_len().saturating_sub(1));
    }
}

impl io::Write for Output<Write> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.push_str(&String::from_utf8_lossy(buf));
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
}
