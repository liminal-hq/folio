// The Hunspell-compatible engine and the bundled dictionary
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::Checker;

/// The English (Canada) dictionary, from SCOWL by way of `wooorm/dictionaries`. Its notice stays
/// beside it (`dictionaries/en-CA/LICENSE`) and is repeated in the third-party notices.
const EN_CA_AFF: &str = include_str!("../dictionaries/en-CA/en_CA.aff");
const EN_CA_DIC: &str = include_str!("../dictionaries/en-CA/en_CA.dic");

#[derive(Debug, thiserror::Error)]
pub enum DictionaryError {
    #[error("the dictionary could not be read: {0}")]
    Parse(String),
}

/// A checker over a Hunspell dictionary.
pub struct SpellbookChecker {
    dictionary: spellbook::Dictionary,
}

impl SpellbookChecker {
    /// A checker from the text of a dictionary's `.aff` and `.dic` files.
    pub fn from_files(aff: &str, dic: &str) -> Result<Self, DictionaryError> {
        spellbook::Dictionary::new(aff, dic)
            .map(|dictionary| Self { dictionary })
            .map_err(|e| DictionaryError::Parse(e.to_string()))
    }

    /// The bundled English (Canada) dictionary.
    pub fn en_ca() -> Result<Self, DictionaryError> {
        Self::from_files(EN_CA_AFF, EN_CA_DIC)
    }
}

impl Checker for SpellbookChecker {
    fn check(&self, word: &str) -> bool {
        self.dictionary.check(word)
    }

    fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        let mut found = Vec::new();
        self.dictionary.suggest(word, &mut found);
        found.truncate(limit);
        found
    }
}
