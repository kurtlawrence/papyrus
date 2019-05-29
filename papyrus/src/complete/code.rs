use super::*;

use racer::{FileCache, Location, Session};

const LIBRS: &str = "lib.rs";

pub struct CodeCompletion;

impl<T: Terminal> Completer<T> for CodeCompletion {
    fn complete(
        &self,
        word: &str,
        prompter: &Prompter<T>,
        start: usize,
        end: usize,
    ) -> Option<Vec<Completion>> {
        let cache = FileCache::default();
        let session = Session::new(&cache);

        session.cache_file_contents(LIBRS, "fn hello() { std::io:: }");

        let v: Vec<_> = racer::complete_from_file(LIBRS, Location::from(22), &session)
            .map(|m| Completion::simple(m.matchstr))
            .collect();

        if v.len() > 0 {
            Some(v)
        } else {
            None
        }
    }
}
