use super::*;

/// These methods are available in _any_ REPL state.
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

    /// The prompt.
    ///
    /// Includes the module name, the editing/mutating state, the command path, the input symbol,
    /// _and the trailing space_. It also includes the colouring byte sequences if specified.
    pub fn prompt(&self, colour: bool) -> String {
        let mod_path = format!("[{}]", self.data.current_mod.display());

        let cmdtree_path = self.data.cmdtree.path();

        let m = if self.data.linking.mutable {
            "-mut"
        } else {
            ""
        };

        let e = if let Some(ei) = self.data.editing {
            format!(
                "-editing-{}{}",
                match ei.editing {
                    Editing::Crate => "crate",
                    Editing::Item => "item",
                    Editing::Stmt => "stmt",
                },
                ei.index
            )
        } else {
            String::new()
        };

        let pcolour = self.data.prompt_colour;

        match (self.more, colour) {
            (true, true) => format!(
                "{} {}{}{}.> ",
                mod_path.color(pcolour),
                cmdtree_path.color(pcolour),
                m.bright_red(),
                e.bright_red()
            ),
            (false, true) => format!(
                "{} {}{}{}=> ",
                mod_path.color(pcolour),
                cmdtree_path.color(pcolour),
                m.bright_red(),
                e.bright_red()
            ),
            (true, false) => format!("{} {}{}{}.> ", mod_path, cmdtree_path, m, e),
            (false, false) => format!("{} {}{}{}=> ", mod_path, cmdtree_path, m, e),
        }
    }
}

impl<S: fmt::Debug, D> fmt::Debug for Repl<S, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Repl in <{:?}> state instance", self.state)
    }
}
