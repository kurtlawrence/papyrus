//! Complete module paths.

use super::*;
use crate::pfh::ModsMap;
use cmdr::ActionArgComplete;
use std::path::{Path, PathBuf};

/// A completer that completes paths to modules, such as the `mod switch` action.
pub struct ModulesCompleter {
    inner: ActionArgComplete,
    mods: Vec<PathBuf>,
}

impl ModulesCompleter {
    /// Build the `ModulesCompleter`.
    pub fn build<T>(cmdr: &cmdtree::Commander<T>, modules: &ModsMap) -> Self {
        let mods = modules
            .iter()
            .map(|x| x.0.clone())
            .collect::<Vec<PathBuf>>();

        let inner = ActionArgComplete::build(&cmdr);

        Self { inner, mods }
    }

    /// Returns the start position of the _last_ word which is broken in context to modules.
    pub fn word_break(line: &str) -> usize {
        word_break_start(line, &[' '])
    }

    /// Get the completions of an actions arguments if it matches the line.
    pub fn complete<'b>(&'b self, line: &'b str) -> Option<impl Iterator<Item = String> + 'b> {
        let actions = ["mod..switch"];

        self.inner
            .find(line, &actions)
            .map(|x| complete_path(x.line, self.mods.iter()))
    }
}

/// Return a set of paths that can be completed using the starting `path`.
fn complete_path<'a, I: 'a + Iterator<Item = P>, P: AsRef<Path>>(
    path: &'a str,
    mods: I,
) -> impl Iterator<Item = String> + 'a {
    mods.filter(move |x| mod_starts_with(x, path)).map(|x| {
        x.as_ref()
            .iter()
            .map(|y| y.to_str().unwrap())
            .fold(String::new(), |mut acc, x| {
                if !acc.is_empty() {
                    acc.push('/');
                }
                acc.push_str(x);
                acc
            })
    })
}

fn mod_starts_with<P: AsRef<Path>>(path: P, line: &str) -> bool {
    // cop the allocation to make matching work on linux and windows
    let line = line.replace("\\", "/");

    let path = path.as_ref();

    if line == " " {
        return true; // we match on a space
    }

    let slashes: &[_] = &['/', '\\'];

    let ends_with_slash = line.ends_with(slashes);

    let line = Path::new(&line);

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
            cmpr(complete_path("o", mods.iter())),
            cmpr2(vec!["one", "one/two", "one/two/three", "own", "own/stuff"])
        );

        assert_eq!(
            cmpr(complete_path("ow", mods.iter())),
            cmpr2(vec!["own", "own/stuff"])
        );

        assert_eq!(
            cmpr(complete_path("on", mods.iter())),
            cmpr2(vec!["one", "one/two", "one/two/three"])
        );

        assert_eq!(
            cmpr(complete_path("one/", mods.iter())),
            cmpr2(vec!["one/two", "one/two/three"])
        );

        assert_eq!(
            cmpr(complete_path("test/", mods.iter())),
            cmpr2(vec!["test/inner", "test/inner/deep", "test/inner/deep2"])
        );

        assert_eq!(
            cmpr(complete_path("test\\inner/", mods.iter())),
            cmpr2(vec!["test/inner/deep", "test/inner/deep2"])
        );

        assert_eq!(
            cmpr(complete_path("test/one", mods.iter())),
            Vec::<String>::new()
        );
    }

    fn cmpr<'a, I: 'a + Iterator<Item = String>>(v: I) -> Vec<String> {
        v.collect()
    }

    fn cmpr2(v: Vec<&str>) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }
}
