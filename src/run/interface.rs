use super::map_xterm_err;
use crate::output::OutputChange;
use crossbeam_channel::{unbounded, Receiver};
use crossterm as xterm;
use std::io::{self, stdout, Stdout, Write};
use xterm::{
    cursor::MoveTo,
    event::{
        Event::{self, *},
        KeyCode::*,
        KeyEvent, KeyModifiers,
    },
    style::Print,
    terminal::{Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};

/// Terminal screen interface.
///
/// It is as its own struct as there is specific configuration and key handling for moving around the
/// interface.
pub struct Screen(Receiver<Event>);

impl Screen {
    pub fn new() -> io::Result<Self> {
        let (tx, rx) = unbounded();
        std::thread::Builder::new()
            .name("terminal-event-buffer".into())
            .spawn(move || loop {
                match xterm::event::poll(std::time::Duration::from_millis(0)) {
                    Ok(true) => {
                        if xterm::event::read()
                            .ok()
                            .and_then(|ev| tx.send(ev).ok())
                            .is_none()
                        {
                            break;
                        }
                    }
                    Ok(false) => {}
                    Err(_) => break,
                }
            })?;
        Ok(Screen(rx))
    }
}

pub struct InputBuffer {
    buf: Vec<char>,
    pos: usize,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            pos: 0,
        }
    }

    pub fn buffer(&self) -> String {
        self.buf.iter().collect()
    }

    /// Character index of cursor.
    pub fn ch_pos(&self) -> usize {
        self.pos
    }

    /// Number of characters.
    pub fn ch_len(&self) -> usize {
        self.buf.len()
    }

    pub fn clear(&mut self) {
        self.buf.clear();
        self.pos = 0;
    }

    pub fn insert(&mut self, ch: char) {
        self.buf.insert(self.pos, ch);
        self.pos += 1;
    }

    pub fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.insert(c);
        }
    }

    /// Removes from _start_ of position.
    pub fn backspace(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
            self.buf.remove(self.pos);
        }
    }

    /// Removes from _end_ of position.
    pub fn delete(&mut self) {
        if self.pos < self.buf.len() {
            self.buf.remove(self.pos);
        }
    }

    /// Return the number moved.
    pub fn move_pos_left(&mut self, n: usize) -> usize {
        let n = if self.pos < n { self.pos } else { n };
        self.pos -= n;
        n
    }

    /// Return the number moved.
    pub fn move_pos_right(&mut self, n: usize) -> usize {
        let max = self.buf.len() - self.pos;
        let n = if n > max { max } else { n };

        self.pos += n;
        n
    }

    pub fn truncate(&mut self, ch_pos: usize) {
        self.buf.truncate(ch_pos);
        if self.pos > self.buf.len() {
            self.pos = self.buf.len()
        }
    }
}

pub struct CItem {
    pub matchstr: String,
    pub input_chpos: usize,
}

#[derive(Default)]
pub struct CompletionWriter {
    input_line: String,
    completions: Vec<CItem>,
    completion_idx: usize,
    lines_to_clear: u16,
}

impl CompletionWriter {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_same_input(&self, line: &str) -> bool {
        self.input_line == line
    }

    pub fn next_completion(&mut self) {
        let idx = self.completion_idx + 1;
        let idx = if idx >= self.completions.len() {
            0
        } else {
            idx
        };
        self.completion_idx = idx;
    }

    pub fn new_completions<I: Iterator<Item = CItem>>(&mut self, completions: I) {
        self.completions.clear();
        for c in completions {
            self.completions.push(c)
        }
        self.completion_idx = 0;
    }

    pub fn overwrite_completion(
        &mut self,
        initial: (u16, u16),
        buf: &mut InputBuffer,
    ) -> io::Result<()> {
        let completion = self.completions.get(self.completion_idx);

        if let Some(CItem {
            matchstr,
            input_chpos,
        }) = completion
        {
            buf.truncate(*input_chpos);
            buf.insert_str(matchstr);
            let (_, y) = write_input_buffer(initial, self.lines_to_clear, &buf)?;
            self.lines_to_clear = y.saturating_sub(initial.1);
            self.input_line = buf.buffer();
        }

        Ok(())
    }
}

pub fn apply_event_to_buf(mut buf: InputBuffer, event: Event) -> (InputBuffer, bool) {
    const NOMOD: KeyModifiers = KeyModifiers::empty();
    macro_rules! nomod {
        ($code:ident) => {
            KeyEvent {
                modifiers: NOMOD,
                code: $code,
            }
        };
    }

    let cmd = match event {
        Key(nomod!(Left)) => {
            buf.move_pos_left(1);
            false
        }
        Key(nomod!(Right)) => {
            buf.move_pos_right(1);
            false
        }
        Key(nomod!(Backspace)) => {
            buf.backspace();
            true
        }
        Key(nomod!(Delete)) => {
            buf.delete();
            true
        }
        Key(KeyEvent {
            modifiers: NOMOD,
            code: Char(c),
        })
        | Key(KeyEvent {
            modifiers: KeyModifiers::SHIFT,
            code: Char(c),
        }) => {
            buf.insert(c);
            true
        }
        _ => false,
    };

    (buf, cmd)
}

/// Given an initial buffer starting point in the terminal, offset the cursor to the buffer's
/// character position. This method is indiscriminant
/// of what is on screen.
///
/// # Wrapping
/// Wrapping on a new line starts at column 0.
pub fn set_cursor_from_input(initial: (u16, u16), buf: &InputBuffer) -> io::Result<()> {
    let (initialx, initialy) = initial;
    let width = xterm::terminal::size()
        .map_err(|e| map_xterm_err(e, "getting cursor failed"))?
        .0 as isize;

    let mut lines = 0;
    let mut chpos = (buf.ch_pos() as isize) - (width - initialx as isize);
    while chpos >= 0 {
        lines += 1;
        chpos -= width;
    }

    chpos = width + chpos;

    let x = chpos as u16;
    let y = initialy + lines;

    stdout()
        .execute(MoveTo(x, y))
        .map(|_| ())
        .map_err(|e| map_xterm_err(e, "cursor setting failed"))
}

/// Returns where the cursor ends up.
pub fn write_input_buffer(
    initial: (u16, u16),
    lines_to_clear: u16,
    buf: &InputBuffer,
) -> io::Result<(u16, u16)> {
    let (x, y) = initial;
    let mut stdout = stdout();
    queue!(stdout, MoveTo(x, y), Clear(ClearType::UntilNewLine))
        .map_err(|e| map_xterm_err(e, ""))?;

    for i in 1..=lines_to_clear {
        queue!(stdout, MoveTo(0, y + i), Clear(ClearType::UntilNewLine))
            .map_err(|e| map_xterm_err(e, ""))?;
    }

    queue!(stdout, MoveTo(x, y), Print(buf.buffer())).map_err(|e| map_xterm_err(e, ""))?;

    stdout.flush()?;

    xterm::cursor::position().map_err(|e| map_xterm_err(e, "failed getting cursor position"))
}

pub fn read_until(
    screen: &mut Screen,
    initial: (u16, u16),
    mut buf: InputBuffer,
    events: &[Event],
) -> (InputBuffer, Event) {
    let reader = &mut screen.0;
    let mut last: Event;
    let mut lines_to_clear = 0;

    loop {
        if let Ok(ev) = reader.recv() {
            last = ev.clone();

            if events.contains(&ev) {
                break;
            }

            buf = {
                let (buf, chg) = apply_event_to_buf(buf, ev);
                if chg {
                    let write_to =
                        write_input_buffer(initial, lines_to_clear, &buf).unwrap_or(initial);
                    lines_to_clear = write_to.1.saturating_sub(initial.1);
                }

                set_cursor_from_input(initial, &buf).ok();

                buf
            };
        } else {
            last = Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: xterm::event::KeyCode::Char('c'),
            });
            break;
        }
    }

    (buf, last)
}

pub fn write_output_chg(change: OutputChange) -> io::Result<()> {
    use OutputChange::*;
    let mut stdout = stdout();
    match change {
        CurrentLine(line) => erase_current_line(stdout)?
            .queue(Print(line))
            .map_err(|e| map_xterm_err(e, ""))
            .and_then(|x| x.flush()),
        NewLine => writeln!(&mut stdout, ""),
    }
}

/// Resets position to start of line.
/// **Does not flush, should be called afterwards.**
pub fn erase_current_line(mut stdout: Stdout) -> io::Result<Stdout> {
    let (_, y) = xterm::cursor::position().expect("failed getting cursor position");
    queue!(stdout, Clear(ClearType::CurrentLine), MoveTo(0, y))
        .map(|_| stdout)
        .map_err(|e| map_xterm_err(e, &line!().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_movement() {
        let mut input = InputBuffer::new();

        "Hello, world!".chars().for_each(|c| input.insert(c));
        assert_eq!(&input.buffer(), "Hello, world!");
        assert_eq!(input.pos, 13);

        // can't go past end of buffer
        input.move_pos_right(1);
        assert_eq!(input.pos, 13);

        input.move_pos_left(1);
        assert_eq!(input.pos, 12);

        input.insert('?');
        assert_eq!(&input.buffer(), "Hello, world?!");
        assert_eq!(input.pos, 13);

        // can't go past start of buffer
        input.move_pos_left(14);
        assert_eq!(input.pos, 0);
    }

    #[test]
    fn test_input_removing() {
        let mut input = InputBuffer::new();

        "Hello, world!".chars().for_each(|c| input.insert(c));

        input.delete();
        assert_eq!(&input.buffer(), "Hello, world!");
        assert_eq!(input.pos, 13);

        input.backspace();
        assert_eq!(&input.buffer(), "Hello, world");
        assert_eq!(input.pos, 12);

        input.move_pos_left(14);
        input.backspace();
        assert_eq!(&input.buffer(), "Hello, world");
        assert_eq!(input.pos, 0);

        input.delete();
        assert_eq!(&input.buffer(), "ello, world");
        assert_eq!(input.pos, 0);
    }
}
