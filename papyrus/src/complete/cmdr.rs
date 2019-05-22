use super::*;
use cmdtree::Commander;

pub struct CmdTreeCompleter {
    /// **Includes** prefix
    space_separated_elements: Vec<String>,
}

impl CmdTreeCompleter {
    pub fn build<T>(cmdr: &Commander<'_, T>, prefix: String) -> Self {
        let space_separated_elements = cmdr
            .structure()
            .into_iter()
            .map(|x| {
                x.split('.')
                    .filter(|x| !x.is_empty())
                    .fold(prefix.clone(), |mut s, x| {
                        if s.len() != prefix.len() {
                            s.push(' ');
                        }
                        s.push_str(x);
                        s
                    })
            })
            .collect();

        Self {
            space_separated_elements,
        }
    }
}

impl<T: Terminal> Completer<T> for CmdTreeCompleter {
    fn complete(
        &self,
        word: &str,
        prompter: &Prompter<T>,
        start: usize,
        end: usize,
    ) -> Option<Vec<Completion>> {
        let line = &prompter.buffer();

        // start is the index in the line
        // need to return just the _word_ portion
        Some(
            self.space_separated_elements
                .iter()
                .filter(|x| x.starts_with(line))
                .map(|x| Completion::simple(x[start..].to_string()))
                .collect(),
        )
    }
}
