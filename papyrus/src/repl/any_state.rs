use super::*;

impl<S, D> Repl<S, D> {
    pub(super) fn move_state<N, F: FnOnce(S) -> N>(self, state_chg: F) -> Repl<N, D> {
        let Repl {
            state,
            data,
            more,
            data_mrker,
        } = self;

        let state = state_chg(state);

        Repl {
            state,
            data,
            more,
            data_mrker,
        }
    }
}

impl<S: fmt::Debug, D> fmt::Debug for Repl<S, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Repl in <{:?}> state instance", self.state)
    }
}
