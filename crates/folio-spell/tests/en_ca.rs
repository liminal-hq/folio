// Tests for the bundled English (Canada) dictionary
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_spell::{find_misspellings, Checker, SpellbookChecker};

fn checker() -> SpellbookChecker {
    SpellbookChecker::en_ca().expect("the bundled dictionary reads")
}

#[test]
fn canadian_spellings_are_right_and_american_ones_are_not() {
    let c = checker();
    for right in [
        "colour",
        "centre",
        "neighbour",
        "realize",
        "cheque",
        "otter",
        "Otter",
    ] {
        assert!(c.check(right), "{right}");
    }
    for wrong in ["color", "center", "neighbor", "otterr"] {
        assert!(!c.check(wrong), "{wrong}");
    }
}

#[test]
fn a_misspelling_is_found_where_it_is_with_suggestions_for_it() {
    let c = checker();
    let text = "Down by the reeds, a small ottr named Pip.";
    let bad = find_misspellings(&c, text, &[]);
    assert_eq!(bad.len(), 1, "{bad:?}");
    let at = text.find("ottr").unwrap();
    assert_eq!((bad[0].start, bad[0].end), (at, at + 4));
    let suggestions = c.suggest("ottr", 5);
    assert!(suggestions.iter().any(|s| s == "otter"), "{suggestions:?}");
    assert!(suggestions.len() <= 5);
}

#[test]
fn a_book_can_ignore_a_word_it_made_up() {
    let c = checker();
    let text = "Pip met Zorblat at the den.";
    let words = |ignore: &[String]| -> Vec<String> {
        find_misspellings(&c, text, ignore)
            .into_iter()
            .map(|m| m.word)
            .collect()
    };
    assert_eq!(words(&[]), ["Zorblat"]);
    assert!(words(&["Zorblat".into()]).is_empty());
}

#[test]
fn contractions_curly_apostrophes_and_numbers_are_not_flagged() {
    let c = checker();
    let bad = find_misspellings(&c, "Don’t forget it's 3rd on page 12, at 5 o’clock.", &[]);
    assert!(bad.is_empty(), "{bad:?}");
}

#[test]
fn a_dictionary_that_is_not_one_is_refused() {
    assert!(SpellbookChecker::from_files("", "not a count\nword").is_err());
}
