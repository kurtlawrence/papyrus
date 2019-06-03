use super::*;

impl Output<Read> {
    pub fn new() -> Self {
        Self {
            state: Read { input_start: 0 },
            buf: String::new(),
            lines_pos: Vec::new(),
        }
    }

    pub fn to_write(self) -> Output<Write> {
        let Output { buf, lines_pos, .. } = self;

        let state = Write;

        Output {
            state,
            buf,
            lines_pos,
        }
    }
}
