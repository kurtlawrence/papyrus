use super::*;

impl Output<Read> {
    pub fn new() -> Self {
        Self {
            state: Read { input_start: 0 },
            buf: String::new(),
            lines_pos: Vec::new(),
            tx: None,
        }
    }

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
}
