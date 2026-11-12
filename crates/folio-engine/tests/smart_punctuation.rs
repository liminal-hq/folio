// Smart punctuation as words are typed and pasted, and in the Book tool, through the engine
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;
use std::sync::Arc;

use folio_engine::{Books, BooksChanged, EditRequest, ExportProgress, Host, PatchEvent};
use folio_engine::{PunctuationRule, ToolFix, ToolReport, ToolRequest};
use folio_model::{
    EditStep, Frame, FrameBase, FrameId, Intent, Pt, Rect, SpreadId, Surface, TableFrame,
    TextInsert, TextPos, TextTarget,
};
use folio_presets::NewBookSpec;

struct Quiet;

impl Host for Quiet {
    fn books_changed(&self, _: &BooksChanged) {}
    fn patched(&self, _: &PatchEvent) {}
    fn export_progress(&self, _: &ExportProgress) {}
    fn home_dir(&self) -> Option<PathBuf> {
        None
    }
    fn document_dir(&self) -> Option<PathBuf> {
        None
    }
    fn version(&self) -> String {
        "test".into()
    }
}

struct Fixture {
    books: Books,
    id: String,
    spread: String,
    frame: FrameId,
}

fn fixture(language: &str, words: &str) -> Fixture {
    let books = Books::new(Arc::new(Quiet));
    let made = books.new_book(&NewBookSpec::default()).unwrap();
    let id = made.summary.id.clone();
    let cover = &made.book.spreads[0];
    let frame = cover
        .frames
        .iter()
        .find(|f| matches!(f, Frame::Text(_)))
        .unwrap()
        .base()
        .id
        .clone();
    books
        .apply(
            &id,
            Intent::SetLanguage {
                language: language.into(),
            },
        )
        .unwrap();
    books
        .apply(
            &id,
            Intent::SetText {
                frame: frame.clone(),
                text: words.into(),
            },
        )
        .unwrap();
    Fixture {
        books,
        id,
        spread: cover.id.0.clone(),
        frame,
    }
}

impl Fixture {
    fn words(&self) -> String {
        let book = self.books.snapshot(&self.id).unwrap().book;
        let Frame::Text(text) = book.spreads[0]
            .frames
            .iter()
            .find(|f| f.base().id == self.frame)
            .unwrap()
        else {
            panic!("a text frame")
        };
        folio_model::text_of(&text.content.as_ref().unwrap().inline)
    }

    fn type_at(&self, offset: u32, text: &str, smart: bool) -> folio_engine::EditAnswer {
        let at = TextPos { para: 0, offset };
        self.books
            .edit_text(
                &self.id,
                EditRequest {
                    target: TextTarget::Frame {
                        frame: self.frame.clone(),
                    },
                    from: at,
                    to: at,
                    insert: TextInsert::Text { text: text.into() },
                    look: None,
                    step: EditStep::Typing,
                    smart: Some(smart),
                },
                &self.spread,
                None,
            )
            .unwrap()
    }
}

#[test]
fn a_second_hyphen_typed_makes_an_em_dash_and_says_what_was_typed() {
    let f = fixture("en-CA", "Wait-");
    let answer = f.type_at(5, "-", true);
    assert_eq!(f.words(), "Wait—");
    assert_eq!(answer.caret, TextPos { para: 0, offset: 5 });
    let note = answer.smart.expect("the words were changed");
    assert_eq!(note.from, TextPos { para: 0, offset: 4 });
    assert_eq!(note.to, TextPos { para: 0, offset: 5 });
    assert_eq!(note.raw, "--");
    assert_eq!(note.text, "—");
    // Putting back what was typed is one more edit.
    f.books
        .edit_text(
            &f.id,
            EditRequest {
                target: TextTarget::Frame {
                    frame: f.frame.clone(),
                },
                from: note.from,
                to: note.to,
                insert: TextInsert::Text { text: note.raw },
                look: None,
                step: EditStep::Own,
                smart: Some(false),
            },
            &f.spread,
            None,
        )
        .unwrap();
    assert_eq!(f.words(), "Wait--");
}

#[test]
fn quotes_follow_the_language_of_the_book() {
    let en = fixture("en", "He said ");
    en.type_at(8, "\"", true);
    assert_eq!(en.words(), "He said “");
    let fr = fixture("fr-CA", "Il dit ");
    fr.type_at(7, "\"", true);
    assert_eq!(fr.words(), "Il dit «\u{a0}");
    let de = fixture("de", "Er sagte ");
    de.type_at(9, "\"", true);
    assert_eq!(de.words(), "Er sagte „");
}

#[test]
fn pasted_plain_words_are_set_and_typing_with_the_setting_off_is_not() {
    let f = fixture("en", "");
    let answer = f.type_at(0, "\"Wait--what...\" it's", true);
    assert_eq!(f.words(), "“Wait—what…” it’s");
    assert_eq!(answer.caret.offset, 17);
    let off = fixture("en", "");
    off.type_at(0, "\"Wait--what...\"", false);
    assert_eq!(off.words(), "\"Wait--what...\"");
}

#[test]
fn the_tool_sets_table_cells_in_the_language_of_the_book() {
    let f = fixture("fr", "");
    let table = Frame::Table(TableFrame::blank(
        FrameBase {
            id: FrameId("fr_table".into()),
            rect: Rect {
                x: Pt(100.0),
                y: Pt(100.0),
                w: Pt(200.0),
                h: Pt(80.0),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
            decorative: false,
        },
        1,
        2,
    ));
    f.books
        .apply(
            &f.id,
            Intent::AddFrame {
                on: Surface::Spread(SpreadId(f.spread.clone())),
                frame: table,
            },
        )
        .unwrap();
    f.books
        .apply(
            &f.id,
            Intent::EditTable {
                frame: FrameId("fr_table".into()),
                edit: folio_model::TableEdit::SetCell {
                    row: 0,
                    col: 1,
                    text: "Il dit \"oui\"--vraiment".into(),
                },
            },
        )
        .unwrap();
    let ToolReport::Punctuation { found } =
        f.books.book_tool(&f.id, &ToolRequest::Punctuation).unwrap()
    else {
        panic!("a punctuation report")
    };
    assert!(found
        .iter()
        .any(|r| r.rule == PunctuationRule::Quotes && r.count == 2));
    f.books
        .book_tool_fix(
            &f.id,
            &ToolFix::Punctuation {
                rules: vec![PunctuationRule::Quotes, PunctuationRule::Dashes],
            },
        )
        .unwrap();
    let book = f.books.snapshot(&f.id).unwrap().book;
    let Some(Frame::Table(t)) = book.spreads[0]
        .frames
        .iter()
        .find(|x| x.base().id.0 == "fr_table")
    else {
        panic!("a table")
    };
    let cell = t.rows[0][1].content.as_ref().unwrap();
    assert_eq!(
        folio_model::text_of(&cell.inline),
        "Il dit «\u{a0}oui\u{a0}»—vraiment"
    );
}

impl Fixture {
    /// Sets the language of the characters `start..end` of the frame's words.
    fn language(&self, start: u32, end: u32, lang: &str) {
        self.books
            .apply(
                &self.id,
                Intent::StyleRange {
                    frame: self.frame.clone(),
                    start,
                    end,
                    change: folio_model::RunChange {
                        lang: Some(lang.into()),
                        ..Default::default()
                    },
                },
            )
            .unwrap();
    }
}

#[test]
fn quotes_follow_the_language_of_the_words_typed_into_before_the_books() {
    // An English book with a French phrase: a quote typed after the phrase is French.
    let f = fixture("en-CA", "Il dit ");
    f.language(0, 7, "fr-CA");
    f.type_at(7, "\"", true);
    assert_eq!(f.words(), "Il dit «\u{a0}");
    // After English words in the same book it is English.
    let g = fixture("en-CA", "He said Il dit ");
    g.language(8, 15, "fr");
    g.type_at(8, "\"", true);
    assert_eq!(g.words(), "He said “Il dit ");
}

#[test]
fn a_language_chosen_for_the_next_words_decides_their_quotes() {
    let f = fixture("en-CA", "Er sagte ");
    let at = TextPos { para: 0, offset: 9 };
    f.books
        .edit_text(
            &f.id,
            EditRequest {
                target: TextTarget::Frame {
                    frame: f.frame.clone(),
                },
                from: at,
                to: at,
                insert: TextInsert::Text { text: "\"".into() },
                look: Some(folio_model::RunChange {
                    lang: Some("de".into()),
                    ..Default::default()
                }),
                step: EditStep::Own,
                smart: Some(true),
            },
            &f.spread,
            None,
        )
        .unwrap();
    assert_eq!(f.words(), "Er sagte „");
}

#[test]
fn the_tool_sets_a_paragraph_in_the_language_all_of_its_words_share() {
    let f = fixture("en-CA", "Il dit \"oui\"");
    f.language(0, 12, "fr");
    f.books
        .book_tool_fix(
            &f.id,
            &ToolFix::Punctuation {
                rules: vec![PunctuationRule::Quotes],
            },
        )
        .unwrap();
    assert_eq!(f.words(), "Il dit «\u{a0}oui\u{a0}»");
}

#[test]
fn spelling_checks_a_stretch_with_the_dictionary_of_its_own_language() {
    // `zorblat` is not English. In a language with no dictionary here it is not checked at all.
    let f = fixture("en-CA", "zorblat zorblat");
    f.language(8, 15, "fr");
    let layout = f
        .books
        .layout_spread(&f.id, &f.spread, Some(&[]))
        .unwrap()
        .layout;
    let frame = layout.frames.iter().find(|l| l.frame == f.frame).unwrap();
    let runs: Vec<_> = frame.lines.iter().flat_map(|l| &l.runs).collect();
    // A long word is hyphenated, so it is several runs.
    let english: Vec<_> = runs.iter().filter(|r| r.lang.is_none()).collect();
    let french: Vec<_> = runs
        .iter()
        .filter(|r| r.lang.as_deref() == Some("fr"))
        .collect();
    assert!(!french.is_empty() && !english.is_empty());
    assert!(
        english.iter().any(|r| !r.miss.is_empty()),
        "the book's words are checked"
    );
    assert!(
        french.iter().all(|r| r.miss.is_empty()),
        "words in a language with no dictionary are not"
    );
    // The proofing list holds only the book's own, and offers no replacement for the other stretch's.
    let listed = f.books.spell_check_book(&f.id, &[]).unwrap();
    assert_eq!(listed.words.len(), 1);
    assert_eq!((listed.words[0].start, listed.words[0].end), (0, 7));
    assert!(!listed.words[0].replaceable, "it has a run with a language");
}
