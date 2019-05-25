//! [`cmdtree`] tree and action completions.
//!
//! [`cmdtree`]: cmdtree

use super::*;
use cmdtree::Commander;

/// Completion items for the [`cmdtree`] class and action structure.
///
/// Implements [`Completer`], as such is _all inclusive_ for completing.
///
/// [`cmdtree`]: cmdtree
/// [`Completer`]: Completer
pub struct TreeCompleter {
    items: Vec<String>,
}

impl TreeCompleter {
    /// Build the `TreeCompleter` from the current state of the `Commander`.
    pub fn build<T>(cmdr: &Commander<T>) -> Self {
        let mut items = cmdtree::completion::create_tree_completion_items(&cmdr);

        items.iter_mut().for_each(|x| {
            if cmdr.at_root() {
                x.insert(0, '.');
            }
        });

        Self { items }
    }
}

impl<T: Terminal> Completer<T> for TreeCompleter {
    fn complete(
        &self,
        _word: &str,
        prompter: &Prompter<T>,
        _start: usize,
        _end: usize,
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

/// Matching of [`cmdtree`] actions.
///
/// [`cmdtree`]: cmdtree
pub struct ActionArgComplete {
    /// All actions available in current state.
    pub items: Vec<cmdtree::completion::ActionMatch>,
}

impl ActionArgComplete {
    /// Build the `ActionArgComplete` with the given `Commander` state.
    pub fn build<T>(cmdr: &Commander<T>) -> Self {
        let mut items = cmdtree::completion::create_action_completion_items(&cmdr);

        items.iter_mut().for_each(|x| {
            if cmdr.at_root() {
                x.match_str.insert(0, '.');
            }
        });

        Self { items }
    }

    /// Determine if the current line matches any available actions. Checks that
    /// the qualified path is within `valid`. Returns the argument scope completion
    /// inputs, such as line, word.
    pub fn find<'a>(&self, line: &'a str, valid: &[&str]) -> Option<ArgComplete<'a>> {
        self.items
            .iter()
            .find(|x| line.starts_with(x.match_str.as_str()))
            .and_then(|x| {
                if valid.contains(&x.qualified_path.as_str()) {
                    let line = &line[x.match_str.len()..];
                    let word_start = linefeed::complete::word_break_start(line, " ");
                    Some(ArgComplete {
                        line,
                        word: &line[word_start..],
                        word_start,
                    })
                } else {
                    None
                }
            })
    }
}

/// The argument scoped inputs for completing.
///
/// The information here is scoped to the argument slice of a line, ie
/// if the line was `some action arg1 arg2` then the `line` would be "arg1 arg2", the
/// `word` would be "arg2" (and if say it was partial it would only return the partial part), and
/// the `word_start` would be 5.
#[derive(Debug, PartialEq)]
pub struct ArgComplete<'a> {
    /// The complete line, scoped to argument section.
    pub line: &'a str,
    /// The final word, broken on spaces.
    pub word: &'a str,
    /// The start index inside `line` of `word`.
    pub word_start: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arg_complete_test() {
        let items = vec![cmdtree::completion::ActionMatch {
            match_str: "some action ".to_string(),
            qualified_path: "some..action".to_string(),
        }];

        let valid = ["some..action"];

        let a = ActionArgComplete { items };

        let f = a.find("some action arg1", &valid).unwrap();

        assert_eq!(
            f,
            ArgComplete {
                line: "arg1",
                word: "arg1",
                word_start: 0
            }
        );

        let f = a.find("some action arg1 argu", &valid).unwrap();

        assert_eq!(
            f,
            ArgComplete {
                line: "arg1 argu",
                word: "argu",
                word_start: 5
            }
        );
    }
}
