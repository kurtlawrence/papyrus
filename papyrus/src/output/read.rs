use super::*;

impl Read {
    pub fn new(current_buffer_len: usize) -> Self {
        Self {
            buf: String::new(),
            prompt_start: current_buffer_len,
            prompt_end: current_buffer_len,
            cursor: 0,
            cursor_start: 0,
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

    /// Insert character into input buffer at cursor position.
    ///
    /// # Panics
    /// Panics if character is `'\n'`. Use `.new_line` function instead.
    ///
    /// # Line Changes
    /// _Always_ triggers a line change event.
    pub fn insert(&mut self, ch: char) {
        if ch == '\n' {
            panic!("use .new_line function to add new line character");
        }

        let buf_pos = self.cursor_pos_in_main_buf();

        self.buf.insert(buf_pos, ch);
        self.state.buf.insert(self.state.cursor, ch);

        self.state.cursor += ch.len_utf8();

        self.send_line_chg(self.lines_len().saturating_sub(1));
    }

    /// Pushes a new line onto the input buffer.
    /// Ignores cursor position, and sets cursor position to end of input buffer.
    pub fn new_line(&mut self) {
        self.push_ch('\n');
        self.state.buf.push('\n');
        self.state.prompt_start = self.buf.len();
        self.state.prompt_end = self.buf.len();
        self.state.cursor = self.state.buf.len();
        self.state.cursor_start = self.state.buf.len();
    }

    /// Moves cursor back a character and erases it.
    /// Only erases to start of current input line.
    ///
    /// # Line Changes
    /// _Always_ triggers a line change event.
    pub fn remove(&mut self) {
        if self.state.cursor_start != self.state.cursor {
            let mut r = 1;

            while !self
                .state
                .buf
                .is_char_boundary(self.state.cursor.saturating_sub(r))
            {
                r += 1;
            }

            self.state.cursor -= r;

            let buf_pos = self.cursor_pos_in_main_buf();

            self.buf.remove(buf_pos);
            self.state.buf.remove(self.state.cursor);

            self.send_line_chg(self.lines_len().saturating_sub(1));
        }
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

    fn cursor_pos_in_main_buf(&self) -> usize {
        let diff = self.state.buf.len().saturating_sub(self.state.cursor);
        let buf_pos = self.buf.len().saturating_sub(diff);
        buf_pos
    }
}
