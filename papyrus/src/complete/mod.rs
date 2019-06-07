//! Completion components and api for aspects of papyrus.

pub mod cmdr;
#[cfg(feature = "racer-completion")]
pub mod code;
pub mod modules;

/// Returns the start position of the _last_ word which is broken by any of the characters.
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
