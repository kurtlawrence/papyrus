use super::*;

impl<S, T: Terminal, D> Repl<S, T, D> {
    /// The current input buffer.
    pub fn input(&self) -> &str {
        self.terminal.input_rdr.input_buffer()
    }

    /// The terminal that the repl reads from and writes to.
    pub fn terminal(&self) -> &T {
        self.terminal.terminal.as_ref()
    }

    pub(super) fn move_state<N, F: FnOnce(S) -> N>(self, state_chg: F) -> Repl<N, T, D> {
        let Repl {
            state,
            terminal,
            data,
            more,
            data_mrker,
        } = self;

        let state = state_chg(state);

        Repl {
            state,
            terminal,
            data,
            more,
            data_mrker,
        }
    }

    /// Set completion on the terminal.
    pub fn set_completion(&mut self, combined: crate::complete::CombinedCompleter<'static, T>) {
        self.terminal
            .input_rdr
            .set_completer(std::sync::Arc::new(combined));
    }
}

impl<S: fmt::Debug, T: Terminal, D> fmt::Debug for Repl<S, T, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Repl in <{:?}> state instance", self.state)
    }
}
