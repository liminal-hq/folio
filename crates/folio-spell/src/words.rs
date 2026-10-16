// Finding the words of a text and the ones a checker does not know
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::Checker;

/// A word in a text, by where it is in UTF-16 code units (`start` up to but not including `end`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordSpan {
    pub start: usize,
    pub end: usize,
    pub word: String,
    /// The same place as a range of bytes of the text, for the one that holds the text in Rust.
    pub bytes: std::ops::Range<usize>,
}

/// A word the checker does not know.
pub type Misspelling = WordSpan;

/// A letter, or a mark that goes with one, for the purpose of finding words.
fn is_letter(c: char) -> bool {
    c.is_alphabetic()
}

/// An apostrophe or a hyphen, which can sit inside a word (don't, o'clock, well-known) but not
/// begin or end one.
fn is_joiner(c: char) -> bool {
    matches!(c, '\'' | '’' | '-' | '‐')
}

/// The words of `text` worth checking, with where they are. A word runs through letters and the
/// joiners between letters; one that has a digit, or that is part of an address or a path, is left
/// out, since a dictionary knows nothing of those.
pub fn words(text: &str) -> Vec<WordSpan> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    // The UTF-16 offset of each char, so a range can be given in the webview's units.
    let mut utf16 = Vec::with_capacity(chars.len() + 1);
    let mut at = 0;
    for (_, c) in &chars {
        utf16.push(at);
        at += c.len_utf16();
    }
    utf16.push(at);

    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !is_letter(chars[i].1) && !chars[i].1.is_ascii_digit() {
            i += 1;
            continue;
        }
        // A run of letters, digits and joiners; trimmed back to letters at both ends afterwards.
        let begin = i;
        while i < chars.len()
            && (is_letter(chars[i].1) || chars[i].1.is_ascii_digit() || is_joiner(chars[i].1))
        {
            i += 1;
        }
        let mut end = i;
        let mut start = begin;
        while start < end && !is_letter(chars[start].1) && !chars[start].1.is_ascii_digit() {
            start += 1;
        }
        while end > start && is_joiner(chars[end - 1].1) {
            end -= 1;
        }
        if start >= end {
            continue;
        }
        let run = &chars[start..end];
        let has_digit = run.iter().any(|(_, c)| c.is_ascii_digit());
        // Part of an address or a path: something between spaces with an `@`, a slash or an
        // underscore in it, or a dot with letters or digits on both sides.
        let address = {
            let mut from = start;
            while from > 0 && !chars[from - 1].1.is_whitespace() {
                from -= 1;
            }
            let mut to = end;
            while to < chars.len() && !chars[to].1.is_whitespace() {
                to += 1;
            }
            let chunk = &chars[from..to];
            chunk
                .iter()
                .any(|(_, c)| matches!(c, '@' | '/' | '\\' | '_'))
                || chunk
                    .windows(3)
                    .any(|w| w[1].1 == '.' && w[0].1.is_alphanumeric() && w[2].1.is_alphanumeric())
        };
        if has_digit || address {
            continue;
        }
        let from = run.first().map_or(0, |(b, _)| *b);
        let to = chars.get(end).map_or(text.len(), |(b, _)| *b);
        out.push(WordSpan {
            start: utf16[start],
            end: utf16[end],
            word: text[from..to].to_owned(),
            bytes: from..to,
        });
    }
    out
}

/// The words of `text` that `checker` does not know and `ignore` does not list. A word the checker
/// does not know in its own case is tried in lower case as well (a capitalised "Otter"), and an
/// apostrophe written as ’ is checked as '.
pub fn find_misspellings(checker: &dyn Checker, text: &str, ignore: &[String]) -> Vec<Misspelling> {
    words(text)
        .into_iter()
        .filter(|w| {
            let plain = w.word.replace('’', "'");
            !(checker.check(&plain)
                || ignore.iter().any(|i| i == &w.word || i == &plain)
                || ignore
                    .iter()
                    .any(|i| i.eq_ignore_ascii_case(&plain) && i == &plain.to_lowercase()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Known(&'static [&'static str]);

    impl Checker for Known {
        fn check(&self, word: &str) -> bool {
            self.0.contains(&word)
        }
        fn suggest(&self, _: &str, _: usize) -> Vec<String> {
            Vec::new()
        }
    }

    fn found(text: &str) -> Vec<(usize, usize, String)> {
        words(text)
            .into_iter()
            .map(|w| (w.start, w.end, w.word))
            .collect()
    }

    #[test]
    fn words_come_with_their_places_in_utf16_units() {
        assert_eq!(
            found("Down by the reeds."),
            [
                (0, 4, "Down".into()),
                (5, 7, "by".into()),
                (8, 11, "the".into()),
                (12, 17, "reeds".into()),
            ]
        );
        // An emoji is two UTF-16 units, so what follows it is two further on than in chars.
        assert_eq!(found("🦦 otter"), [(3, 8, "otter".into())]);
        // An accented word is one word and one unit per letter.
        assert_eq!(found("café au lait")[0], (0, 4, "café".into()));
    }

    #[test]
    fn a_word_is_also_placed_by_bytes_for_whoever_holds_the_text() {
        let text = "🦦 café au lait";
        let all = words(text);
        for w in &all {
            assert_eq!(&text[w.bytes.clone()], w.word);
        }
        assert_eq!(all[0].bytes, 5..10);
    }

    #[test]
    fn apostrophes_and_hyphens_join_a_word_but_do_not_begin_or_end_one() {
        assert_eq!(
            found("don't o’clock well-known 'quoted' and- -dash"),
            [
                (0, 5, "don't".into()),
                (6, 13, "o’clock".into()),
                (14, 24, "well-known".into()),
                (26, 32, "quoted".into()),
                (34, 37, "and".into()),
                (40, 44, "dash".into()),
            ]
        );
    }

    #[test]
    fn numbers_and_addresses_are_not_words() {
        assert!(found("page 12, 3rd, A4 and 1990s")
            .iter()
            .all(|w| w.2 == "page" || w.2 == "and"));
        for text in [
            "write to me@example.com now",
            "see https://example.org/path",
            "a_b and c/d",
            "file.txt",
        ] {
            let words: Vec<_> = found(text).into_iter().map(|w| w.2).collect();
            assert!(
                !words.iter().any(|w| [
                    "me", "example", "com", "https", "org", "path", "a", "b", "c", "d", "file",
                    "txt"
                ]
                .contains(&w.as_str())),
                "{text}: {words:?}"
            );
        }
        // A full stop that ends a sentence does not make the word an address.
        assert_eq!(found("the end.")[1].2, "end");
        assert_eq!(found("the end. Next")[2].2, "Next");
    }

    #[test]
    fn only_unknown_words_are_misspelt_and_ignored_ones_are_not() {
        let checker = Known(&["the", "otter", "don't"]);
        let text = "The otter dont’t sleep, the otter don’t";
        let bad: Vec<_> = find_misspellings(&checker, text, &[])
            .into_iter()
            .map(|m| m.word)
            .collect();
        assert_eq!(bad, ["The", "dont’t", "sleep"]);
        let bad: Vec<_> = find_misspellings(&checker, text, &["sleep".into(), "The".into()])
            .into_iter()
            .map(|m| m.word)
            .collect();
        assert_eq!(bad, ["dont’t"]);
    }
}
