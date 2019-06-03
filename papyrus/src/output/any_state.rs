use super::*;

impl<S> Output<S> {
    /// Full buffer, includes input buffer.
    pub fn buffer(&self) -> &str {
        &self.buf
    }

    pub fn line(&self, idx: usize) -> Option<&str> {
        let lines_len = self.lines_pos.len();

        let end = if idx < lines_len {
            self.lines_pos.get(idx).cloned()
        } else if idx == lines_len {
            Some(self.buf.len())
        } else {
            None
        };

        end.map(|end| {
            let start = if idx == 0 {
                0
            } else {
                self.lines_pos.get(idx - 1).unwrap() + 1
            };

            &self.buf[start..end]
        })
    }

    /// The number of lines. Always >= 1 as the first line, even if empty, counts.
    pub fn lines_len(&self) -> usize {
        self.lines_pos.len() + 1
    }

    pub fn listen(&mut self) -> channel::Receiver<LineChange<'static>> {
        let (tx, rx) = channel::unbounded();

        self.tx = Some(tx);

        rx
    }
}

impl Default for Output<Read> {
    fn default() -> Self {
        Self::new()
    }
}
