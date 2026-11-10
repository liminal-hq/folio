// Times what a keystroke in a very long story costs the layout, and checks the page-count answers
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::Instant;

use folio_layout::{FontBook, Layouter, ParagraphCache, StyleSheet};
use folio_model::{Book, BookKind};
use folio_presets::{new_book, NewBookSpec, Words};

const SENTENCE: &str = "the quick brown fox jumps over a lazy dog while seven small otters watch quietly from the river bank";

/// A novel whose story is `paragraphs` paragraphs of about 150 words each.
fn long(paragraphs: usize) -> (Book, StyleSheet) {
    let made = new_book(&NewBookSpec {
        kind: BookKind::Novel,
        words: Words::Samples,
        ..NewBookSpec::default()
    })
    .unwrap();
    let styles = StyleSheet::from_styles(&made.book.styles);
    let mut book = made.book;
    let one = book.flows[0].paragraphs[0].clone();
    book.flows[0].paragraphs = (0..paragraphs)
        .map(|i| {
            let mut p = one.clone();
            p.inline[0].text = format!("{SENTENCE} {i} {SENTENCE} {SENTENCE} {SENTENCE} {SENTENCE} {SENTENCE} {SENTENCE} {SENTENCE} {SENTENCE}");
            p
        })
        .collect();
    (book, styles)
}

/// What the page-count check after a story edit asks: how many frames more, and how many are used.
fn page_count(
    book: &Book,
    styles: &StyleSheet,
    fonts: &FontBook,
    cache: &ParagraphCache,
) -> (usize, Option<usize>) {
    let flow = book.flows[0].id.clone();
    let fit = Layouter::new(book, styles, fonts)
        .with_paragraph_cache(cache)
        .page_fit(&flow)
        .unwrap()
        .unwrap();
    (fit.needed, Some(fit.used))
}

/// A short story, so this runs in an ordinary test: the single pass agrees with the two answers it
/// replaced, and with and without the cache.
#[test]
fn the_page_fit_is_the_needed_and_used_counts_from_one_pass() {
    let (mut book, styles) = long(30);
    let fonts = FontBook::bundled();
    let cache = ParagraphCache::new();
    let flow = book.flows[0].id.clone();
    for _ in 0..2 {
        for (cached, layouter) in [
            (false, Layouter::new(&book, &styles, &fonts)),
            (
                true,
                Layouter::new(&book, &styles, &fonts).with_paragraph_cache(&cache),
            ),
        ] {
            let fit = layouter.page_fit(&flow).unwrap().unwrap();
            assert_eq!(
                fit.needed,
                layouter.frames_needed(&flow).unwrap(),
                "{cached}"
            );
            assert_eq!(
                Some(fit.used),
                layouter.frames_used(&flow).unwrap(),
                "{cached}"
            );
            assert!(fit.used >= 1);
        }
        // One paragraph changes, and the answers move with it, cache or not.
        book.flows[0].paragraphs[3].inline[0]
            .text
            .push_str(&SENTENCE.repeat(40));
    }
    let first = Layouter::new(&book, &styles, &fonts)
        .page_fit(&flow)
        .unwrap();
    let second = Layouter::new(&book, &styles, &fonts)
        .with_paragraph_cache(&cache)
        .page_fit(&flow)
        .unwrap();
    assert_eq!(first, second);
    assert!(Layouter::new(&book, &styles, &fonts)
        .page_fit(&folio_model::FlowId("nope".into()))
        .unwrap()
        .is_none());
}

/// Characters the body font lacks are set in a fallback whether or not the cache has seen the run
/// before, and a run that needed no split is found again.
#[test]
fn the_cache_splits_runs_for_missing_characters_as_the_layout_does() {
    use folio_layout::paragraph::{Span, TextStyle};
    let fonts = FontBook::bundled();
    let cache = ParagraphCache::new();
    let style = TextStyle::new("Literata", 12.0);
    for text in ["plain words", "漢字 and words", "plain words"] {
        let span = Span::new(text, style.clone());
        let direct = folio_layout::fallback::split_with_fallback(&fonts, &span);
        assert_eq!(cache.split_with_fallback(&fonts, &span), direct, "{text}");
        assert_eq!(
            cache.split_with_fallback(&fonts, &span),
            direct,
            "{text} again"
        );
    }
}

/// Run with `--ignored --nocapture`, ideally with `CARGO_PROFILE_DEV_OPT_LEVEL=3` and one test thread.
#[test]
#[ignore]
fn timing_a_very_long_story() {
    let (mut book, styles) = long(400);
    let words: usize = book.flows[0]
        .paragraphs
        .iter()
        .map(|p| p.text().split_whitespace().count())
        .sum();
    println!(
        "{} paragraphs, {words} words",
        book.flows[0].paragraphs.len()
    );
    let fonts = FontBook::bundled();
    let cache = ParagraphCache::new();
    let first = book
        .spreads
        .iter()
        .find(|s| {
            s.frames
                .iter()
                .any(|f| matches!(f, folio_model::Frame::Text(t) if t.flow.is_some()))
        })
        .unwrap()
        .id
        .clone();

    let t = Instant::now();
    let answer = page_count(&book, &styles, &fonts, &cache);
    println!("page count, cold cache: {:?} -> {answer:?}", t.elapsed());

    book.flows[0].paragraphs[200].inline[0].text.push('x');
    for round in 0..3 {
        book.flows[0].paragraphs[200].inline[0].text.push('x');
        let t = Instant::now();
        let again = page_count(&book, &styles, &fonts, &cache);
        println!(
            "page count after one paragraph changed ({round}): {:?} -> {again:?}",
            t.elapsed()
        );
        let t = Instant::now();
        Layouter::new(&book, &styles, &fonts)
            .with_paragraph_cache(&cache)
            .layout_spread(&first)
            .unwrap()
            .unwrap();
        println!("layout of the spread ({round}): {:?}", t.elapsed());
    }
}

/// The same check once the story has all the frames it needs, which is where it stands for every
/// keystroke after the first: one frame is made tall enough to hold the whole story.
#[test]
#[ignore]
fn timing_the_page_count_of_a_story_that_fits() {
    let (mut book, styles) = long(400);
    for spread in &mut book.spreads {
        for frame in &mut spread.frames {
            if let folio_model::Frame::Text(t) = frame {
                if t.flow.is_some() {
                    t.base.rect.h = folio_model::Pt(400_000.0);
                }
            }
        }
    }
    let fonts = FontBook::bundled();
    let cache = ParagraphCache::new();
    let t = Instant::now();
    let answer = page_count(&book, &styles, &fonts, &cache);
    println!("fitting story, cold cache: {:?} -> {answer:?}", t.elapsed());
    for round in 0..3 {
        book.flows[0].paragraphs[200].inline[0].text.push('x');
        let t = Instant::now();
        let again = page_count(&book, &styles, &fonts, &cache);
        println!(
            "fitting story, one paragraph changed ({round}): {:?} -> {again:?}",
            t.elapsed()
        );
    }
}
