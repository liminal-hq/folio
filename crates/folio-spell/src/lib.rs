// Spell checking: a checker trait, the words of a text, and the dictionaries
//
// Pure Rust with no Tauri. `Checker` is the seam: the first engine is `spellbook`, which reads
// Hunspell `.aff` and `.dic` files, and another can stand in without the callers changing. Ranges
// are in UTF-16 code units, since the webview indexes its strings that way (D-25).
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod checker;
mod words;

pub use checker::{DictionaryError, SpellbookChecker};
pub use words::{find_misspellings, Misspelling, WordSpan};

/// Whether words are spelt right, and what else they might be.
pub trait Checker {
    /// Whether `word` is spelt correctly.
    fn check(&self, word: &str) -> bool;
    /// At most `limit` corrections for `word`, best first.
    fn suggest(&self, word: &str, limit: usize) -> Vec<String>;
}
