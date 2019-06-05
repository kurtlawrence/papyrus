use super::*;

impl Output<Read> {
    /// Construct new, empty output.
    pub fn new() -> Self {
        Self {
            state: Read { input_start: 0 },
            buf: String::new(),
            lines_pos: Vec::new(),
            tx: None,
        }
    }

    /// Finished read state, move to write.
    pub fn to_write(self) -> Output<Write> {
        let Output {
            buf, lines_pos, tx, ..
        } = self;

        let state = Write;

        Output {
            state,
            buf,
            lines_pos,
            tx,
        }
    }

    /// Insert character into input buffer.
    ///
    /// # Line Chanages
    /// _Always_ triggers a line change event.
    pub fn push_input(&mut self, ch: char) {
        if !self.push_ch(ch) {
            self.send_line_chg(self.lines_len().saturating_sub(1))
        }
    }

    /// Returns the current input buffer.
    pub fn input_buffer(&self) -> &str {
        &self.buf[self.state.input_start..]
    }
}
