//! Complete module paths.

use super::*;
use crate::pfh::FileMap;
use cmdr::ActionArgComplete;
use std::path::{Path, PathBuf};

/// A completer that completes paths to modules, such as the `mod switch` action.
///
/// The `Completer` implementation is specific to `papyrus`. If you want to get
/// path completion, see the [`complete_path`] function.
///
/// [`complete_path`]: modules::complete_path
pub struct ModulesCompleter {
    inner: ActionArgComplete,
    mods: Vec<PathBuf>,
}

impl ModulesCompleter {
    /// Build the `ModulesCompleter`.
    pub fn build<T>(cmdr: &cmdtree::Commander<T>, modules: &FileMap) -> Self {
        let mods = modules
            .iter()
            .map(|x| x.0.clone())
            .collect::<Vec<PathBuf>>();

        let inner = ActionArgComplete::build(&cmdr);

        Self { inner, mods }
    }
}

impl<T: Terminal> Completer<T> for ModulesCompleter {
    fn complete(
        &self,
        _word: &str,
        prompter: &Prompter<T>,
        _start: usize,
        _end: usize,
    ) -> Option<Vec<Completion>> {
        let actions = ["mod..switch"];

        let line = prompter.buffer();

        self.inner
            .find(line, &actions)
            .and_then(|x| complete_path(x.line, self.mods.iter()))
    }
}

/// Return a set of paths that can be completed using the starting `path`.
pub fn complete_path<I: Iterator<Item = P>, P: AsRef<Path>>(
    path: &str,
    mods: I,
) -> Option<Vec<Completion>> {
    let path = path.as_ref();

    let v: Vec<_> = mods
        .filter(|x| mod_starts_with(x, path))
        .map(|x| Completion::simple(x.as_ref().display().to_string().replace("\\", "/")))
        .collect();

    if v.len() > 0 {
        Some(v)
    } else {
        None
    }
}

fn mod_starts_with<P: AsRef<Path>>(path: P, line: &str) -> bool {
    let path = path.as_ref();

    if line == " " {
        return true; // we match on a space
    }

    let slashes: &[_] = &['/', '\\'];

    let ends_with_slash = line.ends_with(slashes);

    let line = Path::new(line);

    // can only compare up to line's parent if starts with
    // if line does not have parent then just compare the component idx
    let starts_with = if ends_with_slash {
        path.starts_with(line)
    } else if let Some(parent) = line.parent() {
        path.starts_with(parent)
    } else {
        true
    };

    if starts_with {
        let (line_c, nth) = if ends_with_slash {
            ("", line.iter().count()) // return one above last el
        } else {
            (
                line.iter().last().and_then(|x| x.to_str()).unwrap_or(""),
                line.iter().count().saturating_sub(1),
            )
        };

        path.iter()
            .nth(nth)
            .and_then(|path_c| path_c.to_str())
            .map(|path_c| path_c.starts_with(line_c))
            .unwrap_or(false)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_starts_with_test() {
        assert_eq!(mod_starts_with("hello/world", "hello"), true);
        assert_eq!(mod_starts_with("hello/world", "he"), true);
        assert_eq!(mod_starts_with("hello/world", "hello\\w"), true);
        assert_eq!(mod_starts_with("hello/world", "hello/w"), true);
        assert_eq!(mod_starts_with("hello/world", "hello\\world"), true);

        assert_eq!(mod_starts_with("hello/world", ""), true);
        assert_eq!(mod_starts_with("hello/world", " "), true);

        assert_eq!(mod_starts_with("hello/world", "world"), false);
        assert_eq!(mod_starts_with("hello/world", "hello/hello"), false);
        assert_eq!(mod_starts_with("hello/world", "hello/world/one"), false);

        assert_eq!(mod_starts_with("hello/world", "hello/"), true);
        assert_eq!(mod_starts_with("hello", "hello/"), false);

        assert_eq!(mod_starts_with("own/stuff", "one/"), false);
    }

    #[test]
    fn complete_path_test() {
        let mods: Vec<PathBuf> = vec![
            "one",
            "one/two",
            "one/two/three",
            "own",
            "own/stuff",
            "test",
            "test/inner",
            "test/inner/deep",
            "test/inner/deep2",
        ]
        .into_iter()
        .map(|x| x.into())
        .collect();

        assert_eq!(
            cmpr(&complete_path("o", mods.iter())),
            Some(vec!["one", "one/two", "one/two/three", "own", "own/stuff"])
        );

        assert_eq!(
            cmpr(&complete_path("ow", mods.iter())),
            Some(vec!["own", "own/stuff"])
        );

        assert_eq!(
            cmpr(&complete_path("on", mods.iter())),
            Some(vec!["one", "one/two", "one/two/three"])
        );

        assert_eq!(
            cmpr(&complete_path("one/", mods.iter())),
            Some(vec!["one/two", "one/two/three"])
        );

        assert_eq!(
            cmpr(&complete_path("test/", mods.iter())),
            Some(vec!["test/inner", "test/inner/deep", "test/inner/deep2"])
        );

        assert_eq!(
            cmpr(&complete_path("test\\inner/", mods.iter())),
            Some(vec!["test/inner/deep", "test/inner/deep2"])
        );

        assert_eq!(cmpr(&complete_path("test/one", mods.iter())), None,);
    }

    fn cmpr(v: &Option<Vec<Completion>>) -> Option<Vec<&str>> {
        v.as_ref()
            .map(|x| x.iter().map(|x| x.completion.as_str()).collect())
    }
}
