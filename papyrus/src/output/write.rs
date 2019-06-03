use super::*;
use std::io;

impl Output<Write> {
    pub fn to_read(self) -> Output<Read> {
        let Output { buf, lines_pos, .. } = self;

        let state = Read { input_start: 0 };

        Output {
            state,
            buf,
            lines_pos,
        }
    }

    /// Writes the line contents into the buffer, appended with a `\n` character.
    pub fn write_line(&mut self, line: &str) {
        self.push_str(line);
        self.push_ch('\n');
    }
}

impl io::Write for Output<Write> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.push_str(&String::from_utf8_lossy(buf));
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
