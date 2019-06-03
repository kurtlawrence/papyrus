use super::*;

impl Output<Read> {
    pub fn to_write(self) -> Output<Write> {
        let Output { buf, .. } = self;

        let state = Write;

        Output { state, buf }
    }
}
