use super::*;

impl Default for Output<Read> {
    fn default() -> Self {
        Self {
            state: Read { input_start: 0 },
            buf: String::new(),
        }
    }
}
