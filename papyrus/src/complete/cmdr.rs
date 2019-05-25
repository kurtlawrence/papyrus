use super::*;
use cmdtree::Commander;

pub struct CmdTreeCompleter {
    items: Vec<String>,
}

impl CmdTreeCompleter {
    pub fn build<T>(cmdr: &Commander<T>) -> Self {
        let prefix = if cmdr.at_root() { "." } else { "" };

        let mut items = cmdtree::completion::create_tree_completion_items(&cmdr);

        items.iter_mut().for_each(|x| {
            if cmdr.at_root() {
                x.insert(0, '.');
            }
        });

        Self { items }
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
        let line = prompter.buffer();

        let v: Vec<_> = cmdtree::completion::tree_completions(line, self.items.iter())
            .map(|x| Completion::simple(x.to_string()))
            .collect();

        if v.len() > 0 {
            Some(v)
        } else {
            None
        }
    }
}

pub struct CmdTreeActionCompleter {
    action_elements: Vec<ActionMatch>,
}

impl CmdTreeActionCompleter {
    pub fn build<T>(cmdr: &Commander<T>) -> Self {
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

        Self { action_elements }
    }

    pub fn candidates<'a>(
        &'a self,
        word: &'a str,
        line: &'a str,
        start: usize,
    ) -> impl Iterator<Item = Candidate<'a>> {
        // action match should be unique, such that it is delimited by a space.
        // so if you had myownaction and myotheraction, you won't get a
        // match until 'myownaction ' or 'myotheraction ' is written.
        // this goes for actions with similar prefixes, ie my and myfunc,
        // this would have to match 'my ' and 'myfunc ', which are unique.
        // hence just do a first match and only return _one_ result!

        let candidates = self
            .action_elements
            .iter()
            .filter(move |x| line.starts_with(&x.match_str))
            .map(move |ac| {
                let s = std::cmp::min(ac.match_str.len() + 1, line.len() - 1);

                Candidate {
                    qualified_path: &ac.qualified_path,
                    word,
                    line: &line[s..],
                    word_start: start,
                }
            });

        candidates
    }
}

struct ActionMatch {
    match_str: String,
    qualified_path: String,
}

pub struct Candidate<'a> {
    pub qualified_path: &'a str,
    pub word: &'a str,
    pub line: &'a str,
    pub word_start: usize,
}
