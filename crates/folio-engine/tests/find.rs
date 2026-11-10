// Find and replace over an open book, through the engine: where words are found, and replacing
// them as one undo step
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;
use std::sync::Arc;

use folio_engine::{
    Books, BooksChanged, ExportProgress, FindRequest, Host, MatchRef, PatchEvent, ReplaceRequest,
};
use folio_model::{Frame, Intent, SpreadId, Surface, TextTarget};
use folio_presets::NewBookSpec;
use serde_json::json;

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
    spreads: Vec<String>,
}

fn fixture() -> Fixture {
    let books = Books::new(Arc::new(Quiet));
    let made = books.new_book(&NewBookSpec::default()).unwrap();
    let id = made.summary.id.clone();
    // Start from pages with none of the sample words.
    for spread in &made.book.spreads {
        for frame in &spread.frames {
            books
                .apply(
                    &id,
                    Intent::DeleteFrame {
                        on: Surface::Spread(spread.id.clone()),
                        frame: frame.base().id.clone(),
                    },
                )
                .unwrap();
        }
    }
    let spreads = made.book.spreads.iter().map(|s| s.id.0.clone()).collect();
    Fixture { books, id, spreads }
}

fn text_frame(id: &str, x: f64, y: f64, words: &str) -> Frame {
    serde_json::from_value(json!({
        "type": "text", "id": id, "box": { "x": x, "y": y, "w": 200, "h": 300 },
        "style": "body", "content": { "inline": [{ "text": words }] }
    }))
    .unwrap()
}

impl Fixture {
    fn add(&self, spread: usize, frame: Frame) {
        self.books
            .apply(
                &self.id,
                Intent::AddFrame {
                    on: Surface::Spread(SpreadId(self.spreads[spread].clone())),
                    frame,
                },
            )
            .unwrap();
    }

    fn words(&self, frame: &str) -> String {
        let book = self.books.snapshot(&self.id).unwrap().book;
        let found = book
            .spreads
            .iter()
            .flat_map(|s| &s.frames)
            .find(|f| f.base().id.0 == frame)
            .expect("the frame");
        let Frame::Text(t) = found else {
            panic!("a text frame")
        };
        folio_model::text_of(&t.content.as_ref().unwrap().inline)
    }
}

fn ask(query: &str) -> FindRequest {
    FindRequest {
        query: query.into(),
        match_case: false,
        whole_word: false,
        regex: false,
    }
}

fn replace(query: &str, replacement: &str) -> ReplaceRequest {
    ReplaceRequest {
        find: ask(query),
        replacement: replacement.into(),
        only: None,
        master: None,
    }
}

#[test]
fn a_search_finds_words_on_every_spread() {
    let f = fixture();
    f.add(1, text_frame("a", 20.0, 20.0, "The fox and the hen."));
    f.add(2, text_frame("b", 20.0, 20.0, "Another fox."));
    let report = f.books.find(&f.id, &ask("fox")).unwrap();
    let seen: Vec<_> = report
        .matches
        .iter()
        .map(|m| (m.spread.clone().unwrap(), m.from, m.to))
        .collect();
    assert_eq!(
        seen,
        vec![(f.spreads[1].clone(), 4, 7), (f.spreads[2].clone(), 8, 11)]
    );
    assert!(!report.truncated);
}

#[test]
fn a_story_match_is_on_the_spread_its_words_are_laid_out_on() {
    let f = fixture();
    // A story that fills the first frame, so its last words are in the second, on another spread.
    let filler = "word ".repeat(150);
    f.add(
        1,
        text_frame(
            "first",
            20.0,
            20.0,
            &format!("{filler}\nThe needle is here."),
        ),
    );
    f.add(2, text_frame("second", 20.0, 20.0, ""));
    f.books
        .apply(
            &f.id,
            Intent::LinkFrame {
                frame: folio_model::FrameId("second".into()),
                to: folio_model::FrameId("first".into()),
            },
        )
        .unwrap();
    let report = f.books.find(&f.id, &ask("needle")).unwrap();
    assert_eq!(report.matches.len(), 1);
    let m = &report.matches[0];
    assert!(matches!(m.target, TextTarget::Story { .. }));
    assert_eq!(m.para, 1);
    // The words are past what the first frame holds.
    assert_eq!(m.spread.as_deref(), Some(f.spreads[2].as_str()));
}

#[test]
fn replacing_all_is_one_undo_step() {
    let f = fixture();
    f.add(1, text_frame("a", 20.0, 20.0, "cat and cat\nthe cat"));
    f.add(2, text_frame("b", 20.0, 20.0, "a cat"));
    let report = f.books.replace(&f.id, &replace("cat", "dog")).unwrap();
    assert_eq!(report.replaced, 4);
    assert_eq!(f.words("a"), "dog and dog\nthe dog");
    assert_eq!(f.words("b"), "a dog");
    let depth = f.books.snapshot(&f.id).unwrap();
    assert!(depth.can_undo);
    f.books.undo(&f.id).unwrap();
    assert_eq!(f.words("a"), "cat and cat\nthe cat");
    assert_eq!(f.words("b"), "a cat");
    // The next undo is whatever came before the replace: the frames being added.
    assert_eq!(f.books.find(&f.id, &ask("cat")).unwrap().matches.len(), 4);
    f.books.redo(&f.id).unwrap();
    assert_eq!(f.words("b"), "a dog");
}

#[test]
fn the_replacement_is_set_as_written_and_takes_the_look_of_the_words_it_replaces() {
    let f = fixture();
    let bold: Frame = serde_json::from_value(json!({
        "type": "text", "id": "a", "box": { "x": 20, "y": 20, "w": 200, "h": 300 },
        "style": "body",
        "content": { "inline": [
            { "text": "plain " }, { "text": "loud", "bold": true }, { "text": " plain" }
        ] }
    }))
    .unwrap();
    f.add(1, bold);
    f.books
        .replace(&f.id, &replace("loud", "\"quiet\" --"))
        .unwrap();
    assert_eq!(f.words("a"), "plain \"quiet\" -- plain");
    let book = f.books.snapshot(&f.id).unwrap().book;
    let Some(Frame::Text(t)) = book
        .spreads
        .iter()
        .flat_map(|s| &s.frames)
        .find(|f| f.base().id.0 == "a")
    else {
        panic!("a text frame")
    };
    let runs = &t.content.as_ref().unwrap().inline;
    let bold_words: String = runs
        .iter()
        .filter(|r| r.bold)
        .map(|r| r.text.as_str())
        .collect();
    assert_eq!(bold_words, "\"quiet\" --");
}

#[test]
fn locked_frames_and_other_masters_are_left_alone_and_counted() {
    let f = fixture();
    let mut locked = text_frame("locked", 20.0, 200.0, "a cat");
    if let Frame::Text(t) = &mut locked {
        t.base.locked = true;
    }
    f.add(1, text_frame("free", 20.0, 20.0, "a cat"));
    f.add(1, locked);
    let report = f.books.replace(&f.id, &replace("cat", "dog")).unwrap();
    assert_eq!(
        (
            report.replaced,
            report.skipped_locked,
            report.skipped_masters
        ),
        (1, 1, 0)
    );
    assert_eq!(f.words("free"), "a dog");
    assert_eq!(f.words("locked"), "a cat");
}

#[test]
fn one_match_is_replaced_by_naming_it() {
    let f = fixture();
    f.add(1, text_frame("a", 20.0, 20.0, "cat cat cat"));
    let mut request = replace("cat", "dog");
    request.only = Some(MatchRef {
        target: TextTarget::Frame {
            frame: folio_model::FrameId("a".into()),
        },
        para: 0,
        from: 4,
        to: 7,
    });
    let report = f.books.replace(&f.id, &request).unwrap();
    assert_eq!(report.replaced, 1);
    assert_eq!(f.words("a"), "cat dog cat");
    // A match that is no longer there changes nothing.
    let again = f.books.replace(&f.id, &request).unwrap();
    assert_eq!(again.replaced, 0);
    assert_eq!(f.words("a"), "cat dog cat");
}

#[test]
fn a_pattern_that_cannot_be_read_is_refused_and_changes_nothing() {
    let f = fixture();
    f.add(1, text_frame("a", 20.0, 20.0, "cat"));
    let mut request = replace("(", "x");
    request.find.regex = true;
    let error = f.books.replace(&f.id, &request).unwrap_err();
    assert!(error.to_string().contains("pattern"), "{error}");
    assert_eq!(f.words("a"), "cat");
}

#[test]
fn master_page_words_are_replaced_only_on_the_master_being_edited() {
    let f = fixture();
    f.books.apply(&f.id, Intent::EnsureMasters).unwrap();
    let master = f.books.snapshot(&f.id).unwrap().book.master_pages[0]
        .id
        .clone();
    f.books
        .apply(
            &f.id,
            Intent::AddFrame {
                on: Surface::Master(master.clone()),
                frame: text_frame("mf", 20.0, 20.0, "Running head: cat"),
            },
        )
        .unwrap();
    let found = f.books.find(&f.id, &ask("cat")).unwrap();
    assert_eq!(found.matches.len(), 1);
    assert_eq!(found.matches[0].master.as_deref(), Some(master.0.as_str()));
    let skipped = f.books.replace(&f.id, &replace("cat", "dog")).unwrap();
    assert_eq!((skipped.replaced, skipped.skipped_masters), (0, 1));
    let mut editing = replace("cat", "dog");
    editing.master = Some(master.0.clone());
    let done = f.books.replace(&f.id, &editing).unwrap();
    assert_eq!(done.replaced, 1);
    let book = f.books.snapshot(&f.id).unwrap().book;
    let Frame::Text(t) = &book.master_pages[0].frames.last().unwrap() else {
        panic!("a text frame")
    };
    assert_eq!(
        folio_model::text_of(&t.content.as_ref().unwrap().inline),
        "Running head: dog"
    );
}
