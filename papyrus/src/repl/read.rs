use super::*;

impl<D> Default for Repl<Read, D> {
    fn default() -> Self {
        let mut data = ReplData::default();

        data.redirect_on_execution = false;

        let mut r = Repl {
            state: Read {
                output: Output::default(),
            },

            data: data,
            more: false,
            data_mrker: PhantomData,
        };

        r.draw_prompt();

        r
    }
}

impl<D> Repl<Read, D> {
    /// Overwrite the current line in the input buffer.
    ///
    /// A line is considered if more input is required, the previous input stacked.
    /// Only overwrites the most recent buffer.
    pub fn line_input(&mut self, input: &str) {
        self.state.output.replace_line_input(input);
    }

    /// The current input buffer.
    pub fn input_buffer(&self) -> &str {
        self.state.output.input_buffer()
    }

    /// Read the current contents of the input buffer.
    /// This may move the repl into an evaluating state.
    pub fn read(mut self) -> ReadResult<D> {
        let treat_as_cmd = !self.data.cmdtree.at_root();

        let result = crate::input::determine_result(
            self.state.output.input_buffer(),
            self.state.output.input_buf_line(),
            treat_as_cmd,
        );

        // have to push after as can't take mutable brw and last line
        // if done before will not register cmds
        self.state.output.new_line();

        if result == InputResult::More {
            self.more = true;
            self.draw_prompt();
            ReadResult::Read(self)
        } else {
            self.more = false;
            ReadResult::Eval(self.move_state(|s| Evaluate {
                output: s.output.to_write(),
                result,
            }))
        }
    }

    /// The prompt.
    pub fn prompt(&self) -> String {
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

    pub(super) fn draw_prompt(&mut self) {
        self.state.output.set_prompt_and_trigger(&self.prompt());
    }

    /// Begin listening to line change events on the output.
    pub fn output_listen(&mut self) -> output::Receiver {
        self.state.output.listen()
    }

    /// Close the sender side of the output channel.
    pub fn close_channel(&mut self) {
        self.state.output.close()
    }
}
