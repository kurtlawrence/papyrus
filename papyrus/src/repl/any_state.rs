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

    /// The source code as a string which is being edited.
    ///
    /// This is helpful if an alteration has been requested and you want to
    /// show the old source code.
    pub fn editing_src(&self) -> Option<String> {
        self.data.editing.and_then(|ei| {
            let src = self.data.current_src();

            match ei.editing {
                Editing::Crate => src.crates.get(ei.index).map(|x| &x.src_line).cloned(),
                Editing::Item => src.items.get(ei.index).cloned(),
                Editing::Stmt => src.stmts.get(ei.index).map(|x| x.src_line()),
            }
        })
    }
}

impl<S: fmt::Debug, D> fmt::Debug for Repl<S, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Repl in <{:?}> state instance", self.state)
    }
}
