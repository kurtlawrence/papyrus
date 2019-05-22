use super::*;

pub struct CombinedCompleter {
    pub cmd_tree_completer: cmdr::CmdTreeCompleter,
}

impl<T: Terminal> Completer<T> for CombinedCompleter {
    fn complete(
        &self,
        word: &str,
        prompter: &Prompter<T>,
        start: usize,
        end: usize,
    ) -> Option<Vec<Completion>> {
        self.cmd_tree_completer.complete(word, prompter, start, end)
    }
}
