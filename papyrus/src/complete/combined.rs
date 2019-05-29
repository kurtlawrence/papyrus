use super::*;

/// A collection of completers to be used with the repl.
pub struct CombinedCompleter<'a, T: 'a> {
    /// The completers.
    pub completers: Vec<Box<dyn Completer<T> + 'a>>,
}

impl<'a, T> Completer<T> for CombinedCompleter<'a, T>
where
    T: 'a + Terminal,
{
    fn complete(
        &self,
        word: &str,
        prompter: &Prompter<T>,
        start: usize,
        end: usize,
    ) -> Option<Vec<Completion>> {
        let mut v = Vec::new();

        for completer in self.completers.iter() {
            if let Some(vec) = completer.complete(word, prompter, start, end) {
                v.extend(vec)
            };
        }

        if v.len() > 0 {
            Some(v)
        } else {
            None
        }
    }
}
