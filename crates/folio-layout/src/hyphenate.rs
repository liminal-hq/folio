// Hyphenation: where a word may be broken across lines, by language
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::OnceLock;

use hyphenation::{Hyphenator, Language, Load, Standard};

/// Words shorter than this are never hyphenated.
pub const MIN_WORD_CHARS: usize = 6;

/// Which dictionary a language tag uses. Canadian English follows the US patterns and Canadian
/// French the French ones. A language with no dictionary has none, and its words stay whole.
fn dictionary_language(tag: &str) -> Option<Language> {
    let lower = tag.to_ascii_lowercase();
    let primary = lower.split(['-', '_']).next().unwrap_or("");
    match primary {
        "en" if lower.starts_with("en-gb")
            || lower.starts_with("en-au")
            || lower.starts_with("en-nz") =>
        {
            Some(Language::EnglishGB)
        }
        "en" => Some(Language::EnglishUS),
        "fr" => Some(Language::French),
        _ => None,
    }
}

fn dictionary(language: Language) -> &'static Standard {
    static EN_US: OnceLock<Standard> = OnceLock::new();
    static EN_GB: OnceLock<Standard> = OnceLock::new();
    static FR: OnceLock<Standard> = OnceLock::new();
    let cell = match language {
        Language::EnglishGB => &EN_GB,
        Language::French => &FR,
        _ => &EN_US,
    };
    cell.get_or_init(|| Standard::from_embedded(language).expect("the dictionaries are embedded"))
}

/// The byte offsets in `word` where it may be broken, ascending. `word` is letters only (no
/// punctuation or spaces). Empty for a short word, a word with no letters, or a language with no
/// dictionary. A word that already holds a soft hyphen (U+00AD) breaks only there.
pub fn hyphenation_points(word: &str, language: &str) -> Vec<usize> {
    if word.chars().count() < MIN_WORD_CHARS
        || !word.chars().all(char::is_alphabetic) && !word.contains('\u{ad}')
    {
        return Vec::new();
    }
    let Some(language) = dictionary_language(language) else {
        return Vec::new();
    };
    let mut points = dictionary(language).hyphenate(word).breaks;
    points.retain(|&at| at > 0 && at < word.len() && word.is_char_boundary(at));
    points
}
