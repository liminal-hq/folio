// Smart punctuation: straight quotes, double hyphens and three dots set the way a book sets them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Each rule is applied on its own, to one paragraph of words at a time, so the tool can say how
//! many places each would change and what they would look like before anything is changed.
//! Characters already set the way a book sets them are left alone.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One thing the tool tidies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum PunctuationRule {
    /// Straight quotes and apostrophes become curly ones facing the right way.
    Quotes,
    /// `--` becomes an em dash, and a hyphen with a space either side an en dash.
    Dashes,
    /// Three dots become one ellipsis character.
    Ellipses,
    /// Two or more spaces in a row between words become one.
    Spaces,
}

/// Every rule, in the order they are listed.
pub const RULES: [PunctuationRule; 4] = [
    PunctuationRule::Quotes,
    PunctuationRule::Dashes,
    PunctuationRule::Ellipses,
    PunctuationRule::Spaces,
];

/// `text` with `rule` applied, and how many places it changed.
pub fn apply(text: &str, rule: PunctuationRule) -> (String, usize) {
    match rule {
        PunctuationRule::Quotes => quotes(text),
        PunctuationRule::Dashes => dashes(text),
        PunctuationRule::Ellipses => ellipses(text),
        PunctuationRule::Spaces => spaces(text),
    }
}

/// `text` with every rule in `rules` applied, in the order they are listed.
pub fn smarten(text: &str, rules: &[PunctuationRule]) -> String {
    let mut out = text.to_owned();
    for rule in RULES {
        if rules.contains(&rule) {
            out = apply(&out, rule).0;
        }
    }
    out
}

fn opens_after(c: Option<char>) -> bool {
    match c {
        None => true,
        Some(c) => c.is_whitespace() || "([{—–-“‘".contains(c),
    }
}

fn quotes(text: &str) -> (String, usize) {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut changed = 0;
    for (i, &c) in chars.iter().enumerate() {
        let before = i.checked_sub(1).map(|j| chars[j]);
        let after = chars.get(i + 1).copied();
        match c {
            '"' => {
                out.push(if opens_after(before) { '“' } else { '”' });
                changed += 1;
            }
            '\'' => {
                // An apostrophe, or a quote that closes, follows a letter; one that opens follows
                // a space, and faces the words that follow it, unless a number does (’90s).
                let opens = opens_after(before) && after.is_some_and(char::is_alphabetic);
                out.push(if opens { '‘' } else { '’' });
                changed += 1;
            }
            other => out.push(other),
        }
    }
    (out, changed)
}

fn dashes(text: &str) -> (String, usize) {
    let mut changed = 0;
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '-' && chars.get(i + 1) == Some(&'-') {
            // Any run of hyphens past two is one dash too.
            while chars.get(i) == Some(&'-') {
                i += 1;
            }
            out.push('—');
            changed += 1;
        } else if chars[i] == '-' && i > 0 && chars[i - 1] == ' ' && chars.get(i + 1) == Some(&' ')
        {
            out.push('–');
            changed += 1;
            i += 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    (out, changed)
}

fn ellipses(text: &str) -> (String, usize) {
    let mut changed = 0;
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("...") {
        out.push_str(&rest[..at]);
        out.push('…');
        changed += 1;
        rest = &rest[at + 3..];
    }
    out.push_str(rest);
    (out, changed)
}

fn spaces(text: &str) -> (String, usize) {
    let mut changed = 0;
    let mut out = String::with_capacity(text.len());
    let mut run = 0;
    let mut seen_word = false;
    for c in text.chars() {
        if c == ' ' {
            run += 1;
            // Spaces that start a paragraph are an indent someone chose.
            if !seen_word || run == 1 {
                out.push(c);
            } else if run == 2 {
                changed += 1;
            }
        } else {
            run = 0;
            seen_word = true;
            out.push(c);
        }
    }
    (out, changed)
}

/// A few words around the first place `rule` would change `text`, as they are now and as they would
/// be, or `None` when it would change nothing.
pub fn example(text: &str, rule: PunctuationRule) -> Option<(String, String)> {
    let (after, changed) = apply(text, rule);
    if changed == 0 {
        return None;
    }
    let before: Vec<char> = text.chars().collect();
    let new: Vec<char> = after.chars().collect();
    let at = before
        .iter()
        .zip(new.iter())
        .position(|(a, b)| a != b)
        .unwrap_or(0);
    // Take about 14 characters either side of where they first differ, from the start of a word.
    let from = at.saturating_sub(14);
    let slice = |chars: &[char], end: usize| -> String {
        let end = end.min(chars.len());
        let start = from.min(end);
        chars[start..end].iter().collect()
    };
    Some((slice(&before, at + 14), slice(&new, at + 14)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_face_the_way_they_are_read() {
        let (out, n) = quotes("\"Don't go,\" she said. 'Really,' he said. It's the '90s.");
        assert_eq!(
            out,
            "“Don’t go,” she said. ‘Really,’ he said. It’s the ’90s."
        );
        assert_eq!(n, 7);
        // Curly ones already there are left alone.
        assert_eq!(quotes("“Hello,” she said.").1, 0);
        // A quote after a dash opens, and one after a bracket too.
        assert_eq!(quotes("He said—\"no\" (\"yes\")").0, "He said—“no” (“yes”)");
    }

    #[test]
    fn two_hyphens_are_an_em_dash_and_a_spaced_hyphen_an_en_dash() {
        assert_eq!(dashes("Wait--what?"), ("Wait—what?".to_owned(), 1));
        assert_eq!(dashes("Wait---what?"), ("Wait—what?".to_owned(), 1));
        assert_eq!(dashes("Pages 4 - 6"), ("Pages 4 – 6".to_owned(), 1));
        // A hyphen in a word, and a minus at the start of a line, are not dashes.
        assert_eq!(dashes("well-known").1, 0);
        assert_eq!(dashes("-5 degrees").1, 0);
    }

    #[test]
    fn three_dots_are_one_ellipsis() {
        assert_eq!(ellipses("Well... maybe..."), ("Well… maybe…".to_owned(), 2));
        assert_eq!(ellipses("Well… maybe").1, 0);
    }

    #[test]
    fn spaces_collapse_between_words_but_an_indent_stays() {
        assert_eq!(spaces("one  two   three"), ("one two three".to_owned(), 2));
        assert_eq!(
            spaces("    indented  line"),
            ("    indented line".to_owned(), 1)
        );
    }

    #[test]
    fn rules_apply_in_order_and_only_those_asked_for() {
        let text = "\"Wait--what...\"  she said";
        assert_eq!(smarten(text, &RULES), "“Wait—what…” she said".to_owned());
        assert_eq!(
            smarten(text, &[PunctuationRule::Ellipses]),
            "\"Wait--what…\"  she said".to_owned()
        );
    }

    #[test]
    fn an_example_shows_the_first_place_a_rule_would_change() {
        let (before, after) = example(
            "The long walk home ended with a \"hello\" at the gate.",
            PunctuationRule::Quotes,
        )
        .unwrap();
        assert!(before.contains('"') && after.contains('“'));
        assert_eq!(
            example("Nothing to do here.", PunctuationRule::Quotes),
            None
        );
    }
}
