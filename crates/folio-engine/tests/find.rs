// Finding words in an open book, through the engine: where they are, on which spread
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;
use std::sync::Arc;

use folio_engine::{Books, BooksChanged, ExportProgress, FindRequest, Host, PatchEvent};
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
}

fn ask(query: &str) -> FindRequest {
    FindRequest {
        query: query.into(),
        match_case: false,
        whole_word: false,
        regex: false,
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
fn a_pattern_that_cannot_be_read_is_refused() {
    let f = fixture();
    f.add(1, text_frame("a", 20.0, 20.0, "cat"));
    let mut request = ask("(");
    request.regex = true;
    let error = f.books.find(&f.id, &request).unwrap_err();
    assert!(error.to_string().contains("pattern"), "{error}");
}

#[test]
fn words_on_a_master_page_are_found_with_the_master_and_no_spread() {
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
    f.add(1, text_frame("a", 20.0, 20.0, "cat"));
    let found = f.books.find(&f.id, &ask("cat")).unwrap();
    assert_eq!(found.matches.len(), 2);
    assert_eq!(
        found.matches[0].spread.as_deref(),
        Some(f.spreads[1].as_str())
    );
    assert_eq!(found.matches[1].master.as_deref(), Some(master.0.as_str()));
    assert_eq!(found.matches[1].spread, None);
    assert_eq!((found.matches[1].from, found.matches[1].to), (14, 17));
}

#[test]
fn a_locked_frame_is_found_and_marked() {
    let f = fixture();
    let mut locked = text_frame("locked", 20.0, 200.0, "a cat");
    if let Frame::Text(t) = &mut locked {
        t.base.locked = true;
    }
    f.add(1, locked);
    let found = f.books.find(&f.id, &ask("cat")).unwrap();
    assert!(found.matches[0].locked);
}
