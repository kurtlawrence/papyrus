use super::map_xterm_err;
use crate::output::OutputChange;
use crossbeam_channel::{unbounded, Receiver};
use crossterm as xterm;
use std::{
    fmt,
    io::{self, stdout, Stdout, Write},
};
use xterm::{
    cursor::*,
    event::{
        Event::{self, *},
        KeyCode::*,
        KeyEvent, KeyModifiers,
    },
    style::Print,
    terminal::{enable_raw_mode, Clear, ClearType},
    Result as XResult,
};

pub struct Screen(pub(super) Receiver<Event>);

impl Screen {
    pub fn new() -> io::Result<Self> {
        let (tx, rx) = unbounded();
        std::thread::Builder::new()
            .name("terminal-event-buffer".into())
            .spawn(move || loop {
                match xterm::event::poll(std::time::Duration::from_millis(5)) {
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

    pub fn begin_interface_input<'a>(
        &'a mut self,
        preallocated_buf: &'a mut InputBuffer,
    ) -> XResult<InputInterface<'a>> {
        enable_raw_mode()?;
        preallocated_buf.clear();
        Ok(InputInterface {
            screen: self,
            stdout: io::stdout(),
            buf: preallocated_buf,
            prev_lines_covered: 0,
            prompt_len: 0,
        })
    }
}

pub struct InputInterface<'a> {
    screen: &'a mut Screen,
    stdout: Stdout,
    buf: &'a mut InputBuffer,
    prompt_len: usize,
    prev_lines_covered: u16,
}

impl<'a> InputInterface<'a> {
    pub fn set_prompt(&mut self, prompt: &str) {
        self.buf.move_start();
        for _ in 0..self.prompt_len {
            self.buf.delete();
        }
        self.prompt_len = prompt.chars().count();
        self.buf.insert_str(prompt);
        self.buf.move_end();
    }

    pub fn write<T: fmt::Display>(&mut self, text: T) {
        self.buf.insert_str(&text.to_string());
    }

    pub fn writeln<T: fmt::Display>(&mut self, text: T) {
        self.write(text);
        self.write("\n");
    }

    pub fn flush_buffer(&mut self) -> XResult<()> {
        overwrite_text(0, self.prev_lines_covered, &self.buf)?;
        self.prev_lines_covered = self.buf.covered_lines(term_width_nofail()) as u16;
        Ok(())
    }
}

#[derive(Clone)]
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

    pub fn clear(&mut self) {
        self.buf.clear();
        self.pos = 0;
    }

    pub fn buffer(&self) -> String {
        self.buf.iter().collect()
    }

    /// Number of characters.
    pub fn ch_len(&self) -> usize {
        self.buf.len()
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

    pub fn move_start(&mut self) {
        self.pos = 0;
    }

    pub fn move_end(&mut self) {
        self.pos = self.buf.len();
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

    pub fn covered_lines(&self, width: usize) -> usize {
        let mut lines = 0;
        let mut counter = 0;
        let mut in_esc_seq = false;
        for ch in self.buf.iter().copied() {
            match ch {
                'm' if in_esc_seq => in_esc_seq = false,
                '\u{1b}' if !in_esc_seq => in_esc_seq = true,
                _ if in_esc_seq => (),
                '\n' => {
                    lines += 1;
                    counter = 0
                }
                '\r' => counter = 0,
                '\t' => {
                    let r = counter % 4;
                    counter += 4 - r;
                }
                _ => counter += 1,
            }

            if counter >= width {
                lines += 1;
                counter = 0;
            }
        }
        lines
    }
}

impl fmt::Display for InputBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ch in self.buf.iter().copied() {
            write!(f, "{}", ch)?;
            if ch == '\n' {
                write!(f, "\r")?;
            }
        }
        Ok(())
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
            let prev_lines_covered =
                lines_covered(initial.0 as usize, term_width_nofail(), buf.ch_len());
            buf.truncate(*input_chpos);
            buf.insert_str(matchstr);
            let buf = buf.buffer();
            overwrite_text(
                initial.0 + 1,
                prev_lines_covered.saturating_sub(1) as u16,
                &buf,
            )
            .ok();
            self.input_line = buf;
        }

        Ok(())
    }
}

fn apply_event_to_buf(mut buf: InputBuffer, event: Event) -> (InputBuffer, bool) {
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

fn overwrite_text<T: fmt::Display + Clone>(
    initialx: u16,
    lines_covered: u16,
    text: T,
) -> xterm::Result<()> {
    let mut stdout = stdout();
    // still moves up if lines covered is zero, unsure if crossterm bug and might be changed
    if lines_covered > 0 {
        for _ in 0..lines_covered {
            queue!(stdout, Clear(ClearType::CurrentLine), MoveUp(1))?;
        }
    }
    queue!(
        stdout,
        MoveToColumn(initialx),
        Clear(ClearType::UntilNewLine),
        Print(text)
    )?;

    stdout.flush().map_err(xterm::ErrorKind::IoError)
}

pub fn read_until(
    screen: &mut Screen,
    initial: (u16, u16),
    mut buf: InputBuffer,
    events: &[Event],
) -> (InputBuffer, Event) {
    let reader = &mut screen.0;
    let mut last = Event::Key(KeyEvent {
        modifiers: KeyModifiers::CONTROL,
        code: xterm::event::KeyCode::Char('c'),
    });

    while let Ok(ev) = reader.recv() {
        last = ev;
        if events.contains(&ev) {
            break;
        }

        let prev_lines_covered =
            lines_covered(initial.0 as usize, term_width_nofail(), buf.ch_len());
        let (newbuf, chg) = apply_event_to_buf(buf, ev);
        if chg {
            overwrite_text(
                initial.0 + 1,
                prev_lines_covered.saturating_sub(1) as u16,
                &newbuf,
            )
            .ok();
        }
        buf = newbuf
    }

    (buf, last)
}

/// Returns the number of lines the written text accounts for
pub fn write_output_chg(current_lines_covered: u16, change: OutputChange) -> io::Result<u16> {
    use OutputChange::*;
    let mut stdout = stdout();
    match change {
        CurrentLine(line) => {
            for _ in 1..current_lines_covered {
                queue!(stdout, Clear(ClearType::CurrentLine), MoveUp(1))
                    .map_err(|e| map_xterm_err(e, "Clear a line"))?;
            }
            let mut stdout = erase_current_line(stdout)?;
            queue!(stdout, Print(&line)).map_err(|e| map_xterm_err(e, "printing a line"))?;
            stdout.flush()?;
            Ok(lines_covered(0, term_width_nofail(), line.chars().count()) as u16)
        }
        NewLine => writeln!(&mut stdout).map(|_| 1),
    }
}

/// Resets position to start of line.
/// **Does not flush, should be called afterwards.**
pub fn erase_current_line(mut stdout: Stdout) -> io::Result<Stdout> {
    queue!(stdout, Clear(ClearType::CurrentLine), MoveToColumn(0))
        .map(|_| stdout)
        .map_err(|e| map_xterm_err(e, &line!().to_string()))
}

/// Determines the number of lines a text will cover, from the starting postion and a given cell
/// width.
/// Panics if width is zero.
fn lines_covered(starting: usize, width: usize, ch_count: usize) -> usize {
    assert!(width > 0, "width must be greater than zero");

    let chars = ch_count;

    if chars == 0 {
        return 0;
    }

    let lines = chars / width + 1;
    let md = chars % width;
    if md > width.saturating_sub(starting) {
        lines + 1
    } else if md == 0 && starting == 0 {
        lines - 1 // on boundary
    } else {
        lines
    }
}

fn term_width_nofail() -> usize {
    crossterm::terminal::size().unwrap_or((80, 0)).0 as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use colored::*;

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

    #[test]
    fn test_line_covering() {
        assert_eq!(lines_covered(0, 3, "Hello".chars().count()), 2);
        assert_eq!(lines_covered(0, 1, "".chars().count()), 0);
        assert_eq!(lines_covered(3, 3, "hello".chars().count()), 3);
        assert_eq!(lines_covered(5, 3, "hello".chars().count()), 3);
        assert_eq!(lines_covered(0, 5, "hello".chars().count()), 1);
        assert_eq!(lines_covered(1, 5, "hello".chars().count()), 2);
        assert_eq!(lines_covered(2, 3, "hell".chars().count()), 2);
        assert_eq!(lines_covered(2, 3, "hello".chars().count()), 3);
        assert_eq!(lines_covered(0, 3, "HelloHelloHello".chars().count()), 5);

        let mut inputbuf = InputBuffer::new();

        inputbuf.insert_str(&"red".bright_red().on_bright_blue().to_string());
        assert_eq!(inputbuf.covered_lines(1), 3);
        assert_eq!(inputbuf.covered_lines(2), 1);
        assert_eq!(inputbuf.covered_lines(3), 1);
        assert_eq!(inputbuf.covered_lines(4), 0);
    }

    #[test]
    fn verify_terminal_output() {
        verify_terminal_output_inner().expect("should all pass!");
    }

    fn verify_terminal_output_inner() -> XResult<()> {
        use crate::repl::Repl;
        use crossterm::{terminal::*, *};

        let pos = || cursor::position().unwrap();

        // setup terminal
        let (origcols, origrows) = size()?;
        let mut screen = Screen::new()?;
        let mut inputbuf = InputBuffer::new();
        let mut input = screen.begin_interface_input(&mut inputbuf)?;

        let repl: Repl<_, ()> = Repl::default();
        let prompt = repl.prompt(true);
        let stdout = &mut io::stdout();

        input.writeln("---");
        input.writeln("This tests assumptions about cursor movement and clearing of lines. If these assumptions hold true then robust TUI can be created.");
        input.flush_buffer()?;

        let position = pos();
        dbg!(&position);
        assert_eq!(position.0, 0, "Cursor should be sitting at column zero");

        let pos1 = pos();
        for _ in 0..=origcols {
            input.write("a");
        }
        input.flush_buffer()?;
        let pos2 = pos();
        dbg!(pos1, pos2);
        assert_eq!(
            pos2.0, 1,
            "expecting remainder cursor position will be at column 1"
        );

        // reset to new line -- test that colouring doesn't screw things up
        input.writeln("");
        for _ in 0..=origcols {
            input.write("a".bright_red());
        }
        input.flush_buffer();
        dbg!(input.buf.to_string());
        assert_eq!(
            pos().0,
            1,
            "cusor should be at column 1, all red a's above it"
        );
        // retry with another colour
        for _ in 0..origcols {
            input.write("b".bright_blue());
        }
        input.flush_buffer();
        dbg!(input.buf.to_string());
        assert_eq!(
            pos().0,
            1,
            "cusor should be at column 1, all red a's above it"
        );

        // Ensure to reset terminal state
        disable_raw_mode()?;

        Ok(())
    }
}
