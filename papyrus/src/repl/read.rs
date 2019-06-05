use super::*;

use linefeed::terminal::{DefaultTerminal, Terminal};
use std::io;

// TODO add and handle things like Ctrl and stuff. See mortal::Key for more.
// Ctrl + char
// Left, right, pgup, pgdown blah blah

impl<Data> Default for Repl<Read, DefaultTerminal, Data> {
    fn default() -> Self {
        let mut data = ReplData::default();
        data.redirect_on_execution = false;
        let terminal1 =
            linefeed::terminal::DefaultTerminal::new().expect("failed to start default terminal");
        let terminal2 =
            linefeed::terminal::DefaultTerminal::new().expect("failed to start default terminal");
        let r = Repl {
            state: Read {
                output: Output::default(),
            },
            terminal: ReplTerminal {
                terminal: Arc::new(terminal1),
                input_rdr: InputReader::with_term("papyrus", terminal2)
                    .expect("failed to start input reader"),
            },
            data: data,
            more: false,
            data_mrker: PhantomData,
        };

        r.draw_prompt().unwrap();
        r
    }
}

impl<Term: Terminal + Clone, Data> Repl<Read, Term, Data> {
    /// Starts a repl with the specified terminal rather than the default.
    pub fn with_term(terminal: Term) -> Self {
        let data = ReplData::default();
        let terminal2 = terminal.clone();
        let r = Repl {
            state: Read {
                output: Output::default(),
            },
            terminal: ReplTerminal {
                terminal: Arc::new(terminal),
                input_rdr: InputReader::with_term("papyrus", terminal2)
                    .expect("failed to start input reader"),
            },
            data: data,
            more: false,
            data_mrker: PhantomData,
        };

        r.draw_prompt().unwrap();
        r
    }
}

impl<Term: Terminal, Data> Repl<Read, Term, Data> {
    /// Reads input from the input reader until an evaluation phase can begin.
    pub fn read(mut self) -> Repl<Evaluate, Term, Data> {
        unimplemented!();
        // let term = mortal::Terminal::new().unwrap();

        // let treat_as_cmd = !self.data.cmdtree.at_root();

        // loop {
        //     let prompt = self.prompt();

        //     {
        //         let event = term.read_event(None).unwrap().unwrap();

        //         use mortal::Event as ev;
        //         use mortal::Key as k;
        //         match event {
        //             ev::Key(key) => match key {
        //                 k::Char(ch) => {
        //                     self.state.output.push_input(ch);
        //                 }
        //                 x => {
        //                     dbg!(x);
        //                 }
        //             },
        //             x => {
        //                 dbg!(x);
        //             }
        //         }

        //         // use std::io::Read;

        //         // buf.clear();

        //         // dbg!();

        //         // std::io::stdin().read_to_string(&mut buf);

        //         // dbg!(&buf);

        //         // for ch in buf.chars() {
        //         // 	self.state.output.push_input(ch);
        //         // }

        //         // self.state.output.push_input('\n');
        //     }

        //     // let mut buf = [0u8; 4];

        //     // std::io::stdin().read(&mut buf);

        //     // let ch = String::from_utf8_lossy(&buf).chars().first().unwrap();

        //     // self.state.output.push_input(ch);

        //     // let result = self.terminal.input_rdr.read_input(&prompt, treat_as_cmd);
        //     let result = self
        //         .terminal
        //         .input_rdr
        //         .determine_result(self.state.output.input_buffer(), treat_as_cmd);

        //     self.more = match &result {
        //         InputResult::Empty => self.more,
        //         InputResult::More => true,
        //         _ => false,
        //     };

        //     if !self.more {
        //         return self.move_state(|s| Evaluate {
        //             output: s.output.to_write(),
        //             result,
        //         });
        //     }
        // }
    }

    /// Pushes a single character into the repl. If that character finishes a read phase,
    /// an evaluation phase can begin.
    pub fn push_input(self, input: char) -> PushResult<Term, Data> {
        let treat_as_cmd = !self.data.cmdtree.at_root();
        self.handle_ch(input, treat_as_cmd)
    }

    /// Pushes a string into the repl. If a character exists within the string that
    /// initiates an evaluation phase, the method exits early, returning `Ok(repl, remaining)`
    /// where `remaining` is a slice of the original string that was not read.
    /// If no evaluation phases can begin, the result `Err(repl)` will be returned, in the read state.
    pub fn push_input_str<'s>(
        self,
        input: &'s str,
    ) -> Result<(Repl<Evaluate, Term, Data>, &'s str), Repl<Read, Term, Data>> {
        let treat_as_cmd = !self.data.cmdtree.at_root();

        let mut idx = 0;

        let mut result = PushResult::Read(self);
        for ch in input.chars() {
            result = match result {
                PushResult::Read(repl) => repl.handle_ch(ch, treat_as_cmd),
                PushResult::Eval(repl) => return Ok((repl, &input[idx..])),
            };

            idx += 1; // consumed one character
        }

        match result {
            PushResult::Read(r) => Err(r),
            PushResult::Eval(r) => Ok((r, &input[idx..])),
        }
    }

    pub fn input_ch(&mut self, ch: char) {
        self.state.output.push_input(ch);
    }

    pub fn input_str(&mut self, s: &str) {
        s.chars().for_each(|ch| self.input_ch(ch));
    }

    pub fn read2(mut self) -> ReadResult<Term, Data> {
        self.state.output.push_input('\n');

        let treat_as_cmd = !self.data.cmdtree.at_root();

        let result = self
            .terminal
            .input_rdr
            .determine_result(self.state.output.input_buffer(), treat_as_cmd);

        if result == InputResult::More {
            self.more = true;
            self.draw_prompt().expect("should be able to draw prompt?");
            ReadResult::Read(self)
        } else {
            self.more = false;
            ReadResult::Eval(self.move_state(|s| Evaluate {
                output: s.output.to_write(),
                result,
            }))
        }
    }

    fn handle_ch(mut self, ch: char, treat_as_cmd: bool) -> PushResult<Term, Data> {
        let prompt = self.prompt();

        self.state.output.push_input(ch);

        if ch == '\n' {
            let result = self
                .terminal
                .input_rdr
                .determine_result(self.state.output.input_buffer(), treat_as_cmd);

            if result == InputResult::More {
                self.more = true;
                self.draw_prompt().expect("should be able to draw prompt?");
                PushResult::Read(self)
            } else {
                self.more = false;
                PushResult::Eval(self.move_state(|s| Evaluate {
                    output: s.output.to_write(),
                    result,
                }))
            }
        } else {
            PushResult::Read(self)
        }

        // match self
        //     .terminal
        //     .input_rdr
        //     .push_input(&prompt, treat_as_cmd, ch)
        // {
        //     Some(result) => {
        //         if result == InputResult::More {
        //             self.more = true;
        //             self.draw_prompt().expect("should be able to draw prompt?");
        //             PushResult::Read(self)
        //         } else {
        //             self.more = false;
        //             PushResult::Eval(self.move_state(|s| Evaluate {
        //                 output: s.output.to_write(),
        //                 result,
        //             }))
        //         }
        //     }
        //     None => PushResult::Read(self),
        // }
    }

    fn prompt(&self) -> String {
        let mod_path =
            format!("[{}]", self.data.current_file.display()).color(self.data.prompt_colour);
        let cmdtree_path = self.data.cmdtree.path().color(self.data.prompt_colour);
        let m = if self.data.linking.mutable {
            "-mut"
        } else {
            ""
        }
        .bright_red();
        if self.more {
            format!("{} {}{}.> ", mod_path, cmdtree_path, m)
        } else {
            format!("{} {}{}=> ", mod_path, cmdtree_path, m)
        }
    }

    /// Immediately draw the prompt by doing an immediate read step.
    pub(crate) fn draw_prompt(&self) -> io::Result<()> {
        self.terminal.input_rdr.set_prompt(&self.prompt())?;
        self.terminal
            .input_rdr
            .interface
            .read_line_step(Some(std::time::Duration::new(0, 0)))
            .map(|_| ())
    }

    pub fn output_listen(&mut self) -> output::Receiver {
        self.state.output.listen()
    }
}
