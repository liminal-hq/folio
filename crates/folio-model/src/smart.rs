// Smart punctuation: the rules that set straight quotes, double hyphens and three dots the way a book sets them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! One set of rules for typing, pasting, importing and the Book tool. A rule is applied to words
//! going in after some `before` (the paragraph up to the place they go), one character at a time,
//! so a quote knows what it follows and a second hyphen can turn the first into a dash. A rule may
//! therefore rewrite the end of `before` (`rewind` characters of it); the caller replaces those
//! characters and the inserted ones with `Smartened::text`.

/// How quotation marks are set, which follows the language of the words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuoteStyle {
    /// “double” and ‘single’: English, and what any language we have no better rule for gets.
    Curly,
    /// « double » with a non-breaking space inside, and ‘single’: French.
    French,
    /// „double“ and ‚single‘: German.
    German,
}

const NBSP: char = '\u{a0}';

impl QuoteStyle {
    /// The style a language tag (`en`, `en-CA`, `fr-CA`, `de`) is set in.
    pub fn for_language(tag: &str) -> Self {
        let primary = tag.split(['-', '_']).next().unwrap_or("").to_lowercase();
        match primary.as_str() {
            "fr" => Self::French,
            "de" => Self::German,
            _ => Self::Curly,
        }
    }

    fn open_double(self) -> char {
        match self {
            Self::Curly => '“',
            Self::French => '«',
            Self::German => '„',
        }
    }

    fn close_double(self) -> char {
        match self {
            Self::Curly => '”',
            Self::French => '»',
            Self::German => '“',
        }
    }

    fn open_single(self) -> char {
        match self {
            Self::Curly | Self::French => '‘',
            Self::German => '‚',
        }
    }

    /// The single quote that closes, which is also the apostrophe. German closes with ‘, but an
    /// apostrophe is far commoner than a nested quote, so ’ serves both.
    fn close_single(self) -> char {
        '’'
    }
}

/// Which rules to apply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmartRules {
    pub quotes: bool,
    pub dashes: bool,
    pub ellipses: bool,
}

impl SmartRules {
    pub const ALL: SmartRules = SmartRules {
        quotes: true,
        dashes: true,
        ellipses: true,
    };
}

/// Words once the rules have been applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Smartened {
    /// What replaces the last `rewind` characters of `before` together with the inserted words.
    pub text: String,
    /// How many characters of `before` the rules rewrote (a first hyphen made an em dash by the
    /// second, the two dots an ellipsis is made from).
    pub rewind: usize,
    /// How many places were changed.
    pub changed: usize,
    /// What was there before the rules: the `rewind` characters and the inserted words.
    pub raw: String,
}

/// Whether a quote after `c` opens: at the start, after a space, an opening bracket, a dash or
/// another opening quote.
fn opens_after(c: Option<char>) -> bool {
    match c {
        None => true,
        Some(c) => c.is_whitespace() || "([{—–-“‘„‚«".contains(c),
    }
}

/// `inserted` once it is set, as it would read after `before`, every rule applied.
pub fn smarten(before: &str, inserted: &str, style: QuoteStyle) -> Smartened {
    smarten_with(before, inserted, style, SmartRules::ALL)
}

/// As `smarten`, with only the rules given.
pub fn smarten_with(
    before: &str,
    inserted: &str,
    style: QuoteStyle,
    rules: SmartRules,
) -> Smartened {
    // Only the last three characters before the place can matter to a rule.
    let tail: Vec<char> = {
        let all: Vec<char> = before.chars().rev().take(3).collect();
        all.into_iter().rev().collect()
    };
    let base = tail.len();
    let mut buf = tail;
    // The lowest index of `buf` a rule has changed; characters below it are as they were.
    let mut touched = base;
    let mut changed = 0;
    let chars: Vec<char> = inserted.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        let next = chars.get(i + 1).copied();
        let prev = buf.last().copied();
        let before_that = buf.len().checked_sub(2).map(|j| buf[j]);
        match c {
            '"' if rules.quotes => {
                changed += 1;
                if opens_after(prev) {
                    buf.push(style.open_double());
                    if style == QuoteStyle::French {
                        buf.push(NBSP);
                    }
                } else {
                    if style == QuoteStyle::French {
                        // « and » keep a non-breaking space inside; a space already typed becomes one.
                        if prev.is_some_and(char::is_whitespace) {
                            buf.pop();
                            touched = touched.min(buf.len());
                        }
                        buf.push(NBSP);
                    }
                    buf.push(style.close_double());
                }
            }
            '\'' if rules.quotes => {
                changed += 1;
                // An apostrophe, or a quote that closes, follows a letter; one that opens follows a
                // space, and faces the words that follow it, unless a number does (’90s).
                let opens = opens_after(prev) && next.is_none_or(char::is_alphabetic);
                buf.push(if opens {
                    style.open_single()
                } else {
                    style.close_single()
                });
            }
            c if rules.quotes
                && c.is_ascii_digit()
                && buf.len() == base
                && prev == Some(style.open_single())
                && i == 0
                && opens_after(before_that) =>
            {
                // A digit typed straight after an opening quote it did not know the end of: ’90s.
                buf.pop();
                touched = touched.min(buf.len());
                buf.push(style.close_single());
                changed += 1;
                buf.push(c);
            }
            '-' if rules.dashes && prev == Some('-') => {
                buf.pop();
                touched = touched.min(buf.len());
                buf.push('—');
                changed += 1;
            }
            '-' if rules.dashes && prev == Some('—') => {
                // A run of hyphens past two is one dash too.
            }
            ' ' if rules.dashes && prev == Some('-') && before_that == Some(' ') => {
                buf.pop();
                touched = touched.min(buf.len());
                buf.push('–');
                buf.push(' ');
                changed += 1;
            }
            '.' if rules.ellipses && prev == Some('.') && before_that == Some('.') => {
                buf.pop();
                buf.pop();
                touched = touched.min(buf.len());
                buf.push('…');
                changed += 1;
            }
            c => buf.push(c),
        }
    }
    let touched = touched.min(base);
    let rewind = base - touched;
    let text: String = buf[touched..].iter().collect();
    let raw: String = before
        .chars()
        .rev()
        .take(rewind)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .chain(inserted.chars())
        .collect();
    Smartened {
        text,
        rewind,
        changed,
        raw,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A whole paragraph set at once, as a paste or an import does.
    fn whole(text: &str, style: QuoteStyle) -> String {
        smarten("", text, style).text
    }

    /// The paragraph as it reads after each character is typed in turn.
    fn typed(text: &str, style: QuoteStyle) -> String {
        let mut out = String::new();
        for c in text.chars() {
            let s = smarten(&out, &c.to_string(), style);
            let keep = out.chars().count() - s.rewind;
            out = out.chars().take(keep).collect::<String>() + &s.text;
        }
        out
    }

    #[test]
    fn language_tags_choose_the_style() {
        assert_eq!(QuoteStyle::for_language("en"), QuoteStyle::Curly);
        assert_eq!(QuoteStyle::for_language("en-CA"), QuoteStyle::Curly);
        assert_eq!(QuoteStyle::for_language("fr"), QuoteStyle::French);
        assert_eq!(QuoteStyle::for_language("fr-CA"), QuoteStyle::French);
        assert_eq!(QuoteStyle::for_language("de-AT"), QuoteStyle::German);
        assert_eq!(QuoteStyle::for_language("ja"), QuoteStyle::Curly);
    }

    #[test]
    fn quotes_open_at_a_start_and_after_a_space_a_bracket_or_a_dash() {
        let s = QuoteStyle::Curly;
        assert_eq!(whole("\"Hi\"", s), "“Hi”");
        assert_eq!(whole("said \"no\" (\"yes\")", s), "said “no” (“yes”)");
        assert_eq!(whole("He said—\"no\"", s), "He said—“no”");
        assert_eq!(whole("'Really,' he said", s), "‘Really,’ he said");
    }

    #[test]
    fn apostrophes_close_inside_words_and_before_numbers() {
        let s = QuoteStyle::Curly;
        assert_eq!(whole("It's the '90s", s), "It’s the ’90s");
        assert_eq!(typed("It's the '90s", s), "It’s the ’90s");
        assert_eq!(smarten("", "x", s).changed, 0);
    }

    #[test]
    fn a_quote_already_curly_is_left_alone() {
        let s = smarten("", "“Hello,” she said.", QuoteStyle::Curly);
        assert_eq!((s.text.as_str(), s.changed), ("“Hello,” she said.", 0));
    }

    #[test]
    fn two_hyphens_make_an_em_dash_and_a_spaced_hyphen_an_en_dash() {
        let s = QuoteStyle::Curly;
        assert_eq!(whole("Wait--what?", s), "Wait—what?");
        assert_eq!(whole("Wait---what?", s), "Wait—what?");
        assert_eq!(whole("Pages 4 - 6", s), "Pages 4 – 6");
        assert_eq!(typed("Wait--what?", s), "Wait—what?");
        assert_eq!(typed("Wait---what?", s), "Wait—what?");
        assert_eq!(typed("Pages 4 - 6", s), "Pages 4 – 6");
        // A hyphen in a word and a minus at the start are not dashes.
        assert_eq!(whole("well-known -5", s), "well-known -5");
    }

    #[test]
    fn the_second_hyphen_rewrites_the_first() {
        let s = smarten("Wait-", "-", QuoteStyle::Curly);
        assert_eq!((s.text.as_str(), s.rewind, s.raw.as_str()), ("—", 1, "--"));
    }

    #[test]
    fn three_dots_make_an_ellipsis() {
        let s = QuoteStyle::Curly;
        assert_eq!(whole("Well... maybe...", s), "Well… maybe…");
        assert_eq!(typed("Well...", s), "Well…");
        let third = smarten("Well..", ".", s);
        assert_eq!((third.text.as_str(), third.rewind), ("…", 2));
        assert_eq!(whole("Well… maybe", s), "Well… maybe");
    }

    #[test]
    fn only_the_rules_asked_for_are_applied() {
        let only = SmartRules {
            quotes: false,
            dashes: false,
            ellipses: true,
        };
        let s = smarten_with("", "\"Wait--what...\"", QuoteStyle::Curly, only);
        assert_eq!(s.text, "\"Wait--what…\"");
    }

    #[test]
    fn french_sets_guillemets_with_non_breaking_spaces() {
        let s = QuoteStyle::French;
        assert_eq!(whole("Il dit \"oui\"", s), "Il dit «\u{a0}oui\u{a0}»");
        assert_eq!(typed("Il dit \"oui\"", s), "Il dit «\u{a0}oui\u{a0}»");
        assert_eq!(whole("l'été", s), "l’été");
    }

    #[test]
    fn german_sets_low_opening_quotes() {
        let s = QuoteStyle::German;
        assert_eq!(whole("Er sagte \"nein\"", s), "Er sagte „nein“");
        assert_eq!(whole("Er sagte 'nein'", s), "Er sagte ‚nein’");
        assert_eq!(typed("geht's", s), "geht’s");
    }

    #[test]
    fn a_paragraph_break_in_pasted_words_opens_the_next_quote() {
        assert_eq!(whole("\"One\"\n\"Two\"", QuoteStyle::Curly), "“One”\n“Two”");
    }
}
