use super::*;
use cmdtree::Commander;

pub struct CmdTreeCompleter {
    space_separated_elements: Vec<String>,
}

impl CmdTreeCompleter {
    pub fn build<T>(cmdr: &Commander<T>) -> Self {
        let cpath = cmdr.path();

        let prefix = if cmdr.at_root() { "." } else { "" };

        let space_separated_elements = cmdr
            .structure()
            .into_iter()
            .map(|x| {
                x[cpath.len()..].split('.').filter(|x| !x.is_empty()).fold(
                    String::from(prefix),
                    |mut s, x| {
                        if s.len() != prefix.len() {
                            s.push(' ');
                        }
                        s.push_str(x);
                        s
                    },
                )
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

pub type ActionCompletion =
    fn(qualified_path: &str, word: &str, line: &str, word_start: usize) -> Option<Vec<Completion>>;

pub type QualifiedPath = str;
pub type Word = str;
pub type Line = str;
pub type WordStart = usize;

pub struct CmdTreeActionCompleter<A> {
    action_elements: Vec<ActionMatch>,
    completion_fn: A,
}

impl<A> CmdTreeActionCompleter<A>
where
    A: for<'a> Fn(&'a QualifiedPath, &'a Word, &'a Line, WordStart) -> Option<Vec<Completion>>,
{
    pub fn build<T>(cmdr: &Commander<T>, completion_fn: A) -> Self {
        let root_name = cmdr.root_name();

        let cpath = cmdr.path();

        let prefix = if cmdr.at_root() { "." } else { "" };

        let action_elements = cmdr
            .structure()
            .into_iter()
            .filter(|x| x.contains(".."))
            .map(|x| {
                let action_match = x[cpath.len()..].split('.').filter(|x| !x.is_empty()).fold(
                    String::from(prefix),
                    |mut s, x| {
                        if s.len() != prefix.len() {
                            s.push(' ');
                        }
                        s.push_str(x);
                        s
                    },
                );

                let qualified_path = x[root_name.len() + 1..].to_string();

                ActionMatch {
                    match_str: action_match,
                    qualified_path,
                }
            })
            .collect();

        Self {
            action_elements,
            completion_fn,
        }
    }
}

impl<T, A> Completer<T> for CmdTreeActionCompleter<A>
where
    T: Terminal,
    A: for<'a> Fn(&'a QualifiedPath, &'a Word, &'a Line, WordStart) -> Option<Vec<Completion>>
        + Send
        + Sync,
{
    fn complete(
        &self,
        word: &str,
        prompter: &Prompter<T>,
        start: usize,
        end: usize,
    ) -> Option<Vec<Completion>> {
        let line = &prompter.buffer();

        let candidates = self
            .action_elements
            .iter()
            .filter(|x| line.starts_with(&x.match_str));

        let v: Vec<_> = candidates
            .filter_map(|ac| {
                let s = std::cmp::min(ac.match_str.len() + 1, line.len() - 1);
                (self.completion_fn)(&ac.qualified_path, word, &line[s..], start)
            })
            .flatten()
            .collect();

        if v.len() > 0 {
            Some(v)
        } else {
            None
        }
    }
}

struct ActionMatch {
    match_str: String,
    qualified_path: String,
}
