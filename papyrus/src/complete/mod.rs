//! Completion components and api for aspects of papyrus.

pub mod cmdr;
#[cfg(feature = "racer-completion")]
pub mod code;
mod combined;
pub mod modules;

pub use combined::CombinedCompleter;
pub use linefeed::{Completer, Completion, Prompter, Terminal};
