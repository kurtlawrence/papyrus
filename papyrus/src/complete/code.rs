use super::*;

use racer::{BytePos, FileCache, Location, Match, Session};

const LIBRS: &str = "lib.rs";

pub struct CodeCompletion {
    last_code: String,
    split: std::ops::Range<usize>,
}

impl CodeCompletion {
    pub fn build<T>(repl_data: &crate::repl::ReplData<T>) -> Self {
        let (last_code, map) =
            crate::pfh::code::construct_source_code(repl_data.file_map(), repl_data.linking());

        let split = 0..1;

        Self { last_code, split }
    }

    pub fn complete(&self, injection: &str) -> Vec<Match> {
        let cache = FileCache::default();
        let session = Session::new(&cache);

        session.cache_file_contents(LIBRS, "fn hello() { std::io:: }");

        racer::complete_from_file(LIBRS, Location::from(22), &session).collect()
    }

    pub fn inject(&self, injection: &str) -> (String, BytePos) {
        (String::new(), BytePos::ZERO)
    }
}

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
