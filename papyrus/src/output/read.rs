use super::*;

impl Read {
    pub fn new(current_buffer_len: usize) -> Self {
        Self {
            buf: String::new(),
            prompt_start: current_buffer_len,
            prompt_end: current_buffer_len,
        }
    }
}

impl Output<Read> {
    /// Construct new, empty output.
    pub fn new() -> Self {
        Self {
            state: Read::new(0),
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
        let chg_event = self.push_ch(ch);

        if chg_event {
            // update prompt positions
            self.state.prompt_start = self.buf.len();
            self.state.prompt_end = self.buf.len();
        } else {
            self.send_line_chg(self.lines_len().saturating_sub(1));
        }

        self.state.buf.push(ch); // add to input buffer
    }

    /// Returns the current input buffer.
    pub fn input_buffer(&self) -> &str {
        &self.state.buf
    }

    /// Returns last line of the input buffer.
    pub fn input_buf_line(&self) -> &str {
        &self.buf[self.state.prompt_end..]
    }

    /// Sets the prompt text. Will overwrite previous prompt.
    /// Only sets prompt on current line.
    ///
    /// # Line Changes
    /// Does _not_ trigger line change event.
    ///
    /// # Panics
    /// Panics if _prompt_ contains a new line.
    pub fn set_prompt(&mut self, prompt: &str) {
        if prompt.contains('\n') {
            panic!("prompt cannot contain new line character");
        }

        self.buf
            .replace_range(self.state.prompt_start..self.state.prompt_end, prompt);

        self.state.prompt_end = self.state.prompt_start + prompt.len();
    }

    /// Sets the prompt text. Will overwrite previous prompt.
    /// Only sets prompt on current line.
    ///
    /// # Line Changes
    /// _Always_ triggers a line change event.
    ///
    /// # Panics
    /// Panics if _prompt_ contains a new line.
    pub fn set_prompt_and_trigger(&mut self, prompt: &str) {
        self.set_prompt(prompt);
        self.send_line_chg(self.lines_len().saturating_sub(1));
    }
}
