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
}

impl Default for Output<Read> {
    fn default() -> Self {
        Self::new()
    }
}
