use super::*;
use std::io;

impl Output<Write> {
    pub fn to_read(self) -> Output<Read> {
        let Output { buf, .. } = self;

        let state = Read { input_start: 0 };

        Output { state, buf }
    }
}

impl io::Write for Output<Write> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.push_str(&String::from_utf8_lossy(buf));
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
