//! Completion for rust source code using [`racer`].
//!
//! Requires the _racer-completion_ feature.
//!
//! # Rust `std` lib completion.
//! `racer` requires the Rust standard library source code if completion for the Rust standard
//! library is wanted. [As explained here](https://github.com/racer-rust/racer#configuration) the
//! host machine needs to have the Rust source code locally. This can be achieved with `rustup
//! component add rust-src`.
//!
//! # API Usage
//! To increase completion performance, `CodeCache` should be constructed and passed
//! through when completing.
//! The completion of code is done by injecting current input into the latest source code of the
//! `ReplData`.
//! Below is an example of `racer` code completion.
//!
//! ```rust
//! use papyrus::repl::*;
//!
//! // define some source code
//! let src = "#[derive(Default)]
//! struct MyStruct {
//!     string: String,
//!     number: i32
//! }
//!
//! impl MyStruct {
//!     fn new() -> Self {
//!         MyStruct::default()
//!     }
//!
//!     fn number(&self) -> i32 {
//!         self.number
//!     }
//! }";
//!
//! // create the persistent cache
//! let cache = papyrus::complete::code::CodeCache::new().unwrap_or_else(|e| e.0);
//!
//! // build the repl and inject the source code.
//! // have to eval to get source code in REPL data.
//! let mut repl = Repl::default();
//! repl.line_input(src);
//! let repl = match repl.read() {
//!     ReadResult::Eval(e) => e.eval(&mut ()).repl.print().0,
//!     ReadResult::Read(_) => panic!("should have evaluated"),
//! };
//!
//! // build the completer. uses the current REPL data to construct the source code.
//! let cmpltr = papyrus::complete::code::CodeCompleter::build(&repl.data);
//!
//! // inject something to complete
//! let completions = cmpltr.complete("MyStruct::", None, &cache);
//! assert_eq!(completions.get(0).map(|x| x.matchstr.as_str()), Some("new"));
//! assert_eq!(completions.get(1), None);
//!
//! // completions change on the context.
//! let completions = cmpltr.complete("let mystruct = MyStruct; mystruct.", None, &cache);
//! assert_eq!(completions.get(0).map(|x| x.matchstr.as_str()), Some("number")); // field
//! assert_eq!(completions.get(1).map(|x| x.matchstr.as_str()), Some("number")); // function
//! assert_eq!(completions.get(2).map(|x| x.matchstr.as_str()), Some("string")); // field
//! assert_eq!(completions.get(3), None);
//! ```
//!
//! [`racer`]: racer
use super::*;
use std::io;
use std::path::Path;

use racer::{BytePos, FileCache, Location, Match};

const LIBRS: &str = "lib.rs";

/// Completion used for Rust code in the REPL.
pub struct CodeCompleter {
    last_code: String,
    split: std::ops::Range<usize>,
}

impl CodeCompleter {
    /// Build the code completion state. Uses the current repl state.
    pub fn build<T>(repl_data: &crate::repl::ReplData<T>) -> Self {
        let (last_code, map) = crate::code::construct_source_code(
            repl_data.mods_map(),
            repl_data.linking(),
            repl_data.static_files(),
        );

        let split = map.get(repl_data.current_mod()).cloned().unwrap_or(0..0); // return an empty range if this fails

        CodeCompleter { last_code, split }
    }

    /// Returns the start position of the _last_ word which is broken, in context to rust code.
    pub fn word_break(line: &str) -> usize {
        word_break_start(line, &[' ', ':', '.'])
    }

    /// Get completions that would match a string injected into the current repl state.
    pub fn complete(&self, injection: &str, limit: Option<usize>, cache: &CodeCache) -> Vec<Match> {
        let limit = limit.unwrap_or(std::usize::MAX);

        let session = racer::Session::new(&cache.cache, None);

        let (contents, pos) = self.inject(injection);

        session.cache_file_contents(LIBRS, contents);

        racer::complete_from_file(LIBRS, Location::Point(pos), &session)
            .take(limit)
            .collect()
    }

    /// Inject code into the current source code and return the amended code,
    /// along with the byte position to complete from.
    fn inject(&self, injection: &str) -> (String, BytePos) {
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

/// Caching for code.
pub struct CodeCache {
    cache: FileCache,
}

impl CodeCache {
    /// Construct new cache.
    ///
    /// Checks that Rust source code can be found for completion. If not an `Err` is returned with
    /// a message and the code cache. The code cache will still function but there will not be
    /// completion for the Rust library.
    pub fn new() -> Result<Self, (Self, &'static str)> {
        use racer::RustSrcPathError::*;

        let cache = Self {
            cache: FileCache::new(PapyrusCodeFileLoader),
        };

        match racer::get_rust_src_path() {
            Ok(_) => Ok(cache),
            Err(Missing) => Err((cache, "rust source code does not exist")),
            Err(DoesNotExist(_)) => Err((cache, "rust source code path does not exist")),
            Err(NotRustSourceTree(_)) => Err((
                cache,
                "rust source code path does not have valid rustc source",
            )),
        }
    }
}

struct PapyrusCodeFileLoader;

impl racer::FileLoader for PapyrusCodeFileLoader {
    fn load_file(&self, path: &Path) -> io::Result<String> {
        use std::fs::File;
        use std::io::Read;

        // copied from racers implementation and special handling for lib.rs

        if path == Path::new(LIBRS) {
            Ok(String::new())
        } else {
            let mut rawbytes = Vec::new();
            let mut f = File::open(path)?;
            f.read_to_end(&mut rawbytes)?;

            // skip BOM bytes, if present
            if rawbytes.len() > 2 && rawbytes[0..3] == [0xEF, 0xBB, 0xBF] {
                std::str::from_utf8(&rawbytes[3..])
                    .map(|s| s.to_owned())
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
            } else {
                String::from_utf8(rawbytes).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_test() {
        let cc = CodeCompleter {
            last_code: String::from("Hello morld"),
            split: 5..7, // cut out ' m' such that "Hello" and "orld" is it
        };

        let (s, pos) = cc.inject(", w");

        assert_eq!(&s, "Hello, world");
        assert_eq!(pos, BytePos(8));

        let cc = CodeCompleter {
            last_code: String::from("Hello"),
            split: 5..5, // inject to end
        };

        let (s, pos) = cc.inject(", world");

        assert_eq!(&s, "Hello, world");
        assert_eq!(pos, BytePos(12));

        let cc = CodeCompleter {
            last_code: String::from(", world"),
            split: 0..0, // inject at start
        };

        let (s, pos) = cc.inject("Hello");

        assert_eq!(&s, "Hello, world");
        assert_eq!(pos, BytePos(5));

        let cc = CodeCompleter {
            last_code: String::from("Hello, worm"),
            split: 10..11, // cut less than added
        };

        let (s, pos) = cc.inject("ld");

        assert_eq!(&s, "Hello, world");
        assert_eq!(pos, BytePos(12));
    }

    #[test]
    fn complete_test() {
        let cc = CodeCompleter {
            last_code: String::from("fn apple() {} \n\n fn main() {  }"),
            split: 29..29,
        };

        let (s, _) = cc.inject("ap");

        assert_eq!(&s, "fn apple() {} \n\n fn main() { ap }");

        let matches = cc.complete("ap", None, &CodeCache::new().unwrap_or_else(|e| e.0));

        assert_eq!(matches.get(0).map(|x| x.matchstr.as_str()), Some("apple"));
    }

    #[test]
    fn complete_through_repl() {
        use crate::repl::*;

        let src = "#[derive(Default)]
 struct MyStruct {
     string: String,
     number: i32
 }

 impl MyStruct {
     fn new() -> Self {
         MyStruct::default()
     }

     fn number(&self) -> i32 {
         self.number
     }
 }";

        let cache = CodeCache::new().unwrap_or_else(|e| e.0);

        let mut repl = Repl::default();
        repl.line_input(src);
        let repl = match repl.read() {
            ReadResult::Eval(e) => e.eval(&mut ()).repl.print().0,
            ReadResult::Read(_) => panic!("should have evaluated"),
        };
        println!("{}", repl.output());

        let cmpltr = CodeCompleter::build(&repl.data);

        dbg!(&cmpltr.last_code);

        let completions = cmpltr.complete("MyStruct::", None, &cache);

        dbg!(&completions);

        assert_eq!(completions.get(0).map(|x| x.matchstr.as_str()), Some("new"));
        assert_eq!(completions.get(1), None);

        let completions = cmpltr.complete("let mystruct = MyStruct; mystruct.", None, &cache);

        dbg!(&completions);
        assert_eq!(
            completions.get(0).map(|x| x.matchstr.as_str()),
            Some("number")
        ); // field
        assert_eq!(
            completions.get(1).map(|x| x.matchstr.as_str()),
            Some("number")
        ); // function
        assert_eq!(
            completions.get(2).map(|x| x.matchstr.as_str()),
            Some("string")
        ); // field
        assert_eq!(completions.get(3), None);
    }
}
