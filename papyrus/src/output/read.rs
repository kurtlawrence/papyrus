use super::*;

impl Output<Read> {
    /// Construct new, empty output.
    pub fn new() -> Self {
        let state = Read {
            buf: String::new(),
            prompt_start: 0,
            prompt_end: 0,
            lines_idx: 0,
            start: 0,
        };

        Self {
            state,
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

    pub fn replace_line_input(&mut self, input: &str) {
        self.buf.truncate(self.state.prompt_end);
        self.lines_pos.truncate(self.state.lines_idx);
        self.push_str(input);

        self.state.buf.replace_range(self.state.start.., input);
    }

    pub fn new_line(&mut self) {
        self.push_ch('\n');
		self.state.start = self.state.buf.len();
        self.state.lines_idx = self.lines_len();
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
        self.send_line_chg(false);
    }
}
