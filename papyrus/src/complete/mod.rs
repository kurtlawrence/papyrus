//! Completion components and API, for aspects of `papyrus`.
//!
//! Completion is separated across modules and is usually encapsulated using a contextual
//! structure. The completion is generally done in the same manner, various candidates are built on
//! top of the given REPL state, and matches are returned based on an input line.
//!
//! Each module has slightly differing API implementations (which is why there is no trait based
//! approach).

pub mod cmdr;
#[cfg(feature = "racer-completion")]
pub mod code;
pub mod modules;

/// Returns the start position of the _last_ word which is broken by any of the characters.
///
/// # Example
/// ```rust
/// let s = "Hello, world!";
/// let b = papyrus::complete::word_break_start(s, &[' ']);
/// assert_eq!(b, 7);
/// assert_eq!(&s[b..], "world!");
/// ```
pub fn word_break_start(s: &str, word_break_chars: &[char]) -> usize {
    let mut start = s.len();

    for (idx, ch) in s.char_indices().rev() {
        if word_break_chars.contains(&ch) {
            break;
        }
        start = idx;
    }

    start
}
