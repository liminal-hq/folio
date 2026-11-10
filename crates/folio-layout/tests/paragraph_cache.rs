// Checks that laying out with the paragraph cache gives the same lines, and shapes only what changed
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::Instant;

use folio_layout::wire::SpreadLayout;
use folio_layout::{FontBook, Layouter, ParagraphCache, StyleSheet};
use folio_model::{Book, BookKind};
use folio_presets::{new_book, NewBookSpec, Words};

fn novel() -> (Book, StyleSheet) {
    let made = new_book(&NewBookSpec {
        kind: BookKind::Novel,
        words: Words::Samples,
        ..NewBookSpec::default()
    })
    .unwrap();
    let styles = StyleSheet::from_styles(&made.book.styles);
    (made.book, styles)
}

fn all(book: &Book, styles: &StyleSheet, cache: Option<&ParagraphCache>) -> Vec<SpreadLayout> {
    let fonts = FontBook::bundled();
    book.spreads
        .iter()
        .map(|s| {
            let mut layouter = Layouter::new(book, styles, &fonts);
            if let Some(cache) = cache {
                layouter = layouter.with_paragraph_cache(cache);
            }
            layouter.layout_spread(&s.id).unwrap().unwrap()
        })
        .collect()
}

#[test]
fn a_cached_layout_is_the_layout_without_the_cache() {
    let (book, styles) = novel();
    let cache = ParagraphCache::new();
    let plain = all(&book, &styles, None);
    let cold = all(&book, &styles, Some(&cache));
    let warm = all(&book, &styles, Some(&cache));
    assert_eq!(plain, cold);
    assert_eq!(plain, warm);
    assert!(!cache.is_empty());
}

#[test]
fn an_edit_to_one_paragraph_shapes_only_that_paragraph_again() {
    let (mut book, styles) = novel();
    let cache = ParagraphCache::new();
    all(&book, &styles, Some(&cache));
    let kept = cache.len();
    assert!(
        book.flows[0].paragraphs.len() > 3,
        "the sample story has paragraphs"
    );
    book.flows[0].paragraphs[2].inline[0]
        .text
        .push_str(" and one more thing");
    let cached = all(&book, &styles, Some(&cache));
    // Only the paragraph that changed (and the pieces of it that carry over a frame) are new.
    assert!(cache.len() - kept <= 3, "{} new", cache.len() - kept);
    assert_eq!(cached, all(&book, &styles, None));
}

/// How much the cache saves on a long story: run with `--ignored --nocapture`.
#[test]
#[ignore]
fn timing_a_story_with_and_without_the_cache() {
    let (mut book, styles) = novel();
    let text = "the quick brown fox jumps over a lazy dog while seven small otters watch quietly from the river bank";
    let one = book.flows[0].paragraphs[0].clone();
    book.flows[0].paragraphs = (0..400)
        .map(|i| {
            let mut p = one.clone();
            p.inline[0].text = format!("{text} {i} {text} {text}");
            p
        })
        .collect();
    let cache = ParagraphCache::new();
    let t = Instant::now();
    all(&book, &styles, None);
    println!("without the cache: {:?}", t.elapsed());
    all(&book, &styles, Some(&cache));
    book.flows[0].paragraphs[200].inline[0].text.push('x');
    let t = Instant::now();
    all(&book, &styles, Some(&cache));
    println!("warm cache after one paragraph changed: {:?}", t.elapsed());
}
