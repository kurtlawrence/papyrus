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

        let split = map.get(repl_data.current_file()).cloned().unwrap_or(0..0); // return an empty range if this fails

        Self { last_code, split }
    }

    pub fn complete(&self, injection: &str) -> Vec<Match> {
        let cache = FileCache::default();
        let session = Session::new(&cache);

        let (contents, pos) = self.inject(injection);

        session.cache_file_contents(LIBRS, contents);

        racer::complete_from_file(LIBRS, Location::Point(pos), &session).collect()
    }

    pub fn inject(&self, injection: &str) -> (String, BytePos) {
        let cap = self.last_code.len() + self.split.start - self.split.end + injection.len();
        let mut s = String::with_capacity(cap);

        s.push_str(&self.last_code[..self.split.start]);
        s.push_str(injection);
        s.push_str(&self.last_code[self.split.end..]);

        debug_assert_eq!(s.len(), cap);

        let pos = (self.split.start + injection.len()).into();

        (s, pos)
    }
}

impl<T: Terminal> Completer<T> for CodeCompletion {
    fn complete(
        &self,
        _word: &str,
        prompter: &Prompter<T>,
        _start: usize,
        _end: usize,
    ) -> Option<Vec<Completion>> {
        let v: Vec<_> = self
            .complete(prompter.buffer())
            .into_iter()
            .map(|x| Completion::simple(x.matchstr))
            .collect();

        if v.len() > 0 {
            Some(v)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_test() {
        let cc = CodeCompletion {
            last_code: String::from("Hello morld"),
            split: 5..7, // cut out ' m' such that "Hello" and "orld" is it
        };

        let (s, pos) = cc.inject(", w");

        assert_eq!(&s, "Hello, world");
        assert_eq!(pos, BytePos(8));

        let cc = CodeCompletion {
            last_code: String::from("Hello"),
            split: 5..5, // inject to end
        };

        let (s, pos) = cc.inject(", world");

        assert_eq!(&s, "Hello, world");
        assert_eq!(pos, BytePos(12));

        let cc = CodeCompletion {
            last_code: String::from(", world"),
            split: 0..0, // inject at start
        };

        let (s, pos) = cc.inject("Hello");

        assert_eq!(&s, "Hello, world");
        assert_eq!(pos, BytePos(5));

        let cc = CodeCompletion {
            last_code: String::from("Hello, worm"),
            split: 10..11, // cut less than added
        };

        let (s, pos) = cc.inject("ld");

        assert_eq!(&s, "Hello, world");
        assert_eq!(pos, BytePos(12));
    }

    #[test]
    fn complete_test() {
        let cc = CodeCompletion {
            last_code: String::from("fn apple() {} \n\n fn main() {  }"),
            split: 29..29,
        };

        let (s, _) = cc.inject("ap");

        assert_eq!(&s, "fn apple() {} \n\n fn main() { ap }");

        let matches = cc.complete("ap");

        assert_eq!(matches.get(0).map(|x| x.matchstr.as_str()), Some("apple"));
    }
}
