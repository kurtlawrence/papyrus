use crate::output::OutputChange;
use crossterm as xterm;
use mortal::Event;
use mortal::{Event::*, Key::*};
use std::io::{self, stdout, Stdout, Write};
use xterm::{Clear, ClearType, ExecutableCommand, Goto, Output, QueueableCommand};

/// Terminal screen interface.
///
/// Its own struct as there is specific configuration and key handling for moving around the
/// interface.
pub struct Screen(mortal::Screen);

impl Screen {
    pub fn new() -> io::Result<Self> {
        let config = mortal::PrepareConfig {
            block_signals: true,
            ..Default::default()
        };

        Ok(Screen(mortal::Screen::new(config)?))
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
}

pub enum EventAction {
    Left(u16),
    Right(u16),
    InputChange,
    NoAction,
}

/// Waits for a terminal event to occur.
///
/// > Post-processes escaped input to work with WSL.
pub fn read_terminal_event(screen: &Screen) -> io::Result<Event> {
    use mortal::{Event::*, Key::*, Signal::*};

    let screen = &screen.0;

    let ev = screen.read_event(None)?.unwrap_or(NoEvent);

    let res = match ev {
        Key(Escape) => {
            // The escape key can be prelude to arrow keys
            // To handle this we need to get the next _two_
            // events, but they should be fast in coming
            // so timeout and if they don't come, well just return
            // Escape
            const ESC_TIMEOUT: Option<std::time::Duration> =
                Some(std::time::Duration::from_millis(1));

            let fst = screen.read_event(ESC_TIMEOUT)?;
            let snd = screen.read_event(ESC_TIMEOUT)?;

            let ev = match (fst, snd) {
                (Some(fst), Some(snd)) => pat_match_escaped_keys(fst, snd),
                _ => None,
            };

            ev.unwrap_or(Key(Escape))
        }
        Key(Ctrl('c')) => Signal(Interrupt),
        x => x,
    };

    Ok(res)
}

fn pat_match_escaped_keys(first: Event, second: Event) -> Option<Event> {
    use mortal::{Event::*, Key::*};

    match (first, second) {
        (Key(Char('[')), Key(Char('A'))) => Some(Key(Up)),
        (Key(Char('[')), Key(Char('B'))) => Some(Key(Down)),
        (Key(Char('[')), Key(Char('C'))) => Some(Key(Right)),
        (Key(Char('[')), Key(Char('D'))) => Some(Key(Left)),
        _ => None,
    }
}

pub struct TermEventIter<'a>(pub &'a mut Screen);

impl<'a> Iterator for TermEventIter<'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        read_terminal_event(self.0).ok()
    }
}

pub fn apply_event_to_buf(mut buf: InputBuffer, event: Event) -> (InputBuffer, bool) {
    let cmd = match event {
        Key(Left) => {
            buf.move_pos_left(1);
            false
        }
        Key(Right) => {
            buf.move_pos_right(1);
            false
        }
        Key(Char(c)) => {
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
    let term = xterm::terminal();
    let width = term.terminal_size().0 as isize;

    let mut lines = 0;
    let mut chpos = (buf.ch_pos() as isize) - (width - initialx as isize);
    while chpos >= 0 {
        lines += 1;
        chpos -= width;
    }

    chpos = width + chpos;

    let x = chpos as u16;
    let y = initialy + lines;

    xterm::cursor().goto(x, y).map_err(|e| match e {
        xterm::ErrorKind::IoError(e) => e,
        _ => io::Error::new(io::ErrorKind::Other, "cursor setting failed"),
    })
}

/// Returns where the cursor ends up.
pub fn write_input_buffer(
    initial: (u16, u16),
    lines_to_clear: u16,
    buf: &InputBuffer,
) -> io::Result<(u16, u16)> {
    let (x, y) = initial;
    let mut stdout = stdout()
        .queue(Goto(x, y))
        .queue(Clear(ClearType::UntilNewLine));

    for i in 1..=lines_to_clear {
        stdout = stdout
            .queue(Goto(0, y + i))
            .queue(Clear(ClearType::UntilNewLine));
    }

    stdout
        .queue(Goto(x, y))
        .queue(Output(buf.buffer()))
        .flush()?;

    Ok(xterm::cursor().pos())
}

pub fn execute_input_cmd(buf: &InputBuffer, action: EventAction) -> io::Result<()> {
    let stdout = stdout();
    match action {
        EventAction::Left(x) => {
            if x > 0 {
                stdout.execute(xterm::Left(x));
            }
        }
        EventAction::Right(x) => {
            if x > 0 {
                stdout.execute(xterm::Right(x));
            }
        }
        EventAction::InputChange => {
            let offset = buf.ch_pos().saturating_sub(1) as u16;
            let stdout = if offset > 0 {
                stdout.queue(xterm::Left(offset))
            } else {
                stdout
            };

            let stdout = stdout
                .queue(xterm::Clear(xterm::ClearType::UntilNewLine))
                .queue(xterm::Output(buf.buffer()));

            let offset = buf.ch_len().saturating_sub(buf.ch_pos()) as u16;
            if offset > 0 {
                stdout.queue(xterm::Left(offset))
            } else {
                stdout
            }
            .flush()?;
        }
        _ => (),
    };

    Ok(())
}

pub fn read_until(screen: &mut Screen, buf: InputBuffer, events: &[Event]) -> (InputBuffer, Event) {
    let iter = TermEventIter(screen);

    let mut last = Event::NoEvent;

    let initial = xterm::cursor().pos();

    let mut lines_to_clear = 0;

    let input = iter
        .inspect(|ev| last = *ev)
        .take_while(|ev| !events.contains(ev))
        .fold(buf, |buf, ev| {
            let (buf, chg) = apply_event_to_buf(buf, ev);
            if chg {
                let write_to = write_input_buffer(initial, lines_to_clear, &buf).unwrap_or(initial);
                lines_to_clear = write_to.1.saturating_sub(initial.1);
            }

            set_cursor_from_input(initial, &buf).ok();

            buf
        });

    (input, last)
}

pub fn write_output_chg(change: OutputChange) -> io::Result<()> {
    use OutputChange::*;
    let mut stdout = stdout();
    match change {
        CurrentLine(line) => erase_current_line(stdout).queue(Output(line)).flush(),
        NewLine => writeln!(&mut stdout, ""),
    }
}

/// Resets position to start of line.
/// **Does not flush, should be called afterwards.**
pub fn erase_current_line(stdout: Stdout) -> Stdout {
    let (_, y) = xterm::cursor().pos();
    stdout
        .queue(Clear(ClearType::CurrentLine))
        .queue(Goto(0, y))
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
