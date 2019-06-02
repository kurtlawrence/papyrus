use super::*;

impl Output<Write> {
    pub fn to_read(self) -> Output<Read> {
        let Output { state, buf } = self;

        let state = Read { input_start: 0 };

        Output { state, buf }
    }
}
