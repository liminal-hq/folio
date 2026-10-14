// Tests for hyphenation points
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::hyphenate::hyphenation_points;

fn pieces(word: &str, language: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0;
    for at in hyphenation_points(word, language) {
        out.push(word[start..at].to_string());
        start = at;
    }
    out.push(word[start..].to_string());
    out
}

#[test]
fn english_words_break_at_syllables() {
    assert_eq!(pieces("hyphenation", "en-CA"), ["hy", "phen", "a", "tion"]);
    assert!(pieces("beautiful", "en-CA").len() > 1);
}

#[test]
fn canadian_and_american_english_share_patterns_but_british_differs() {
    assert_eq!(
        pieces("hyphenation", "en-US"),
        pieces("hyphenation", "en-CA")
    );
    assert!(hyphenation_points("hyphenation", "en-GB").len() > 1);
}

#[test]
fn french_breaks_with_the_french_patterns() {
    assert!(pieces("extraordinaire", "fr-CA").len() > 1);
    assert_eq!(
        pieces("extraordinaire", "fr-CA"),
        pieces("extraordinaire", "fr-FR")
    );
}

#[test]
fn short_words_and_unknown_languages_stay_whole() {
    assert!(hyphenation_points("otter", "en-CA").is_empty());
    assert!(hyphenation_points("hyphenation", "xx").is_empty());
    assert!(hyphenation_points("hyphenation", "").is_empty());
}

#[test]
fn words_with_digits_or_punctuation_are_left_alone() {
    assert!(hyphenation_points("abc12345", "en-CA").is_empty());
    assert!(hyphenation_points("well-known", "en-CA").is_empty());
}

#[test]
fn a_soft_hyphen_is_the_only_break() {
    let word = "extra\u{ad}ordinary";
    let points = hyphenation_points(word, "en-CA");
    assert_eq!(points.len(), 1);
    assert_eq!(&word[..points[0]], "extra");
}

#[test]
fn points_are_ascending_char_boundaries() {
    for word in ["naïveté", "préparation", "extraordinaire", "Straßenbahnhof"] {
        let points = hyphenation_points(word, "fr-CA");
        assert!(points.windows(2).all(|w| w[0] < w[1]));
        assert!(points.iter().all(|&p| word.is_char_boundary(p)));
    }
}
