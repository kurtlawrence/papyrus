//! Completion components and api for aspects of papyrus.

pub mod cmdr;
mod combined;
pub mod modules;

pub use combined::CombinedCompleter;
pub use linefeed::{Completer, Completion, Prompter, Terminal};
