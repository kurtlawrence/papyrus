use super::*;

impl<S> Output<S> {
    /// Full buffer, includes input buffer.
    pub fn buffer(&self) -> &str {
        &self.buf
    }
}

impl Default for Output<Read> {
    fn default() -> Self {
        Self {
            state: Read { input_start: 0 },
            buf: String::new(),
        }
    }
}
