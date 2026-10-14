// Tests for the Knuth–Plass and first-fit line breakers
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::breaker::{break_lines, first_fit, Break, Item, Params, INFINITE};
use proptest::prelude::*;

/// Words of the given widths, with a space of 4 (stretching 2, shrinking 1) between them.
fn paragraph(words: &[f64]) -> Vec<Item> {
    let mut items = Vec::new();
    for (i, w) in words.iter().enumerate() {
        if i > 0 {
            items.push(Item::glue(4.0, 2.0, 1.0));
        }
        items.push(Item::Box { width: *w });
    }
    items.extend(Item::forced_break());
    items
}

fn lengths(items: &[Item], breaks: &[Break]) -> Vec<f64> {
    let mut out = Vec::new();
    let mut start = 0;
    for b in breaks {
        let mut width = 0.0;
        for item in &items[start..b.at] {
            match item {
                Item::Box { width: w } => width += w,
                Item::Glue { width: w, .. } => width += w,
                Item::Penalty { .. } => {}
            }
        }
        if let Item::Penalty { width: w, .. } = items[b.at] {
            width += w;
        }
        out.push(width);
        start = b.at + 1;
    }
    out
}

fn fixed(width: f64) -> impl Fn(usize) -> f64 {
    move |_| width
}

#[test]
fn one_short_paragraph_is_a_single_line() {
    let items = paragraph(&[20.0, 30.0, 25.0]);
    let broken = break_lines(&items, &fixed(200.0), &Params::default());
    assert_eq!(broken.breaks.len(), 1);
    assert_eq!(broken.breaks[0].at, items.len() - 1);
    assert_eq!(broken.tolerance, Some(1.0));
}

#[test]
fn lines_fit_the_measure() {
    let words = [30.0; 12];
    let items = paragraph(&words);
    let broken = break_lines(&items, &fixed(100.0), &Params::default());
    assert!(broken.breaks.len() >= 4);
    for (b, length) in broken.breaks.iter().zip(lengths(&items, &broken.breaks)) {
        assert!((-1.0..=1.0).contains(&b.ratio) || b.at == items.len() - 1);
        // A line is its natural width, shrunk or stretched by its ratio, to the measure.
        assert!(length <= 100.0 + 3.0, "{length}");
    }
}

#[test]
fn optimal_breaks_beat_first_fit() {
    // First-fit packs the first line full and leaves a very loose second line; Knuth–Plass balances.
    let items = paragraph(&[40.0, 38.0, 22.0, 22.0, 22.0, 22.0, 22.0]);
    let width = 110.0;
    let optimal = break_lines(&items, &fixed(width), &Params::default());
    let greedy = first_fit(&items, &fixed(width));
    let worst = |breaks: &[Break]| {
        breaks[..breaks.len() - 1]
            .iter()
            .map(|b| b.ratio.abs())
            .fold(0.0, f64::max)
    };
    assert!(worst(&optimal.breaks) <= worst(&greedy) + 1e-9);
}

#[test]
fn a_forced_break_ends_the_line_where_it_stands() {
    let mut items = vec![Item::Box { width: 20.0 }];
    items.extend(Item::forced_break());
    items.push(Item::Box { width: 25.0 });
    items.extend(Item::forced_break());
    let broken = break_lines(&items, &fixed(200.0), &Params::default());
    assert_eq!(broken.breaks.len(), 2);
    assert_eq!(broken.breaks[0].at, 2);
}

#[test]
fn the_first_line_can_have_its_own_width() {
    let items = paragraph(&[30.0; 8]);
    let width = |line: usize| if line == 0 { 70.0 } else { 100.0 };
    let broken = break_lines(&items, &width, &Params::default());
    let first = lengths(&items, &broken.breaks)[0];
    assert!(first <= 73.0, "{first}");
}

#[test]
fn a_hyphen_is_used_when_it_helps_and_adds_its_width() {
    // "ex-ample" can break after "ex-"; the other words leave a gap that only the hyphen closes.
    let items = vec![
        Item::Box { width: 50.0 },
        Item::glue(4.0, 2.0, 1.0),
        Item::Box { width: 30.0 },
        Item::Penalty {
            width: 4.0,
            penalty: 50.0,
            flagged: true,
        },
        Item::Box { width: 40.0 },
    ];
    let mut items = items;
    items.extend(Item::forced_break());
    let broken = break_lines(&items, &fixed(92.0), &Params::default());
    assert_eq!(broken.breaks.len(), 2);
    assert_eq!(broken.breaks[0].at, 3, "broke at the hyphen");
}

#[test]
fn a_word_wider_than_the_line_still_breaks_with_first_fit() {
    let items = paragraph(&[300.0, 20.0, 20.0]);
    let broken = break_lines(&items, &fixed(100.0), &Params::default());
    assert_eq!(broken.tolerance, None);
    assert!(broken.breaks.len() >= 2);
    assert_eq!(broken.breaks.last().unwrap().at, items.len() - 1);
}

#[test]
fn a_loose_line_raises_the_tolerance() {
    // Two words that cannot fill a long line without stretching far more than the first try allows.
    let items = paragraph(&[30.0, 30.0, 30.0]);
    let broken = break_lines(&items, &fixed(150.0), &Params::default());
    assert!(broken.tolerance.is_some());
    assert_eq!(broken.breaks.len(), 1);
}

#[test]
fn forbidden_breaks_are_never_taken() {
    let items = vec![
        Item::Box { width: 50.0 },
        Item::Penalty {
            width: 0.0,
            penalty: INFINITE,
            flagged: false,
        },
        Item::glue(4.0, 2.0, 1.0),
        Item::Box { width: 50.0 },
        Item::glue(4.0, 2.0, 1.0),
        Item::Box { width: 50.0 },
        Item::forced_break()[0],
        Item::forced_break()[1],
    ];
    let broken = break_lines(&items, &fixed(110.0), &Params::default());
    assert!(broken.breaks.iter().all(|b| b.at != 1));
}

#[test]
fn empty_paragraphs_break_once() {
    let items: Vec<Item> = Item::forced_break().to_vec();
    let broken = break_lines(&items, &fixed(100.0), &Params::default());
    assert_eq!(broken.breaks.len(), 1);
}

proptest! {
    /// However the words fall, the breaks are strictly ascending, end at the final break, and
    /// every line but an overfull one fits its measure once its glue is adjusted.
    #[test]
    fn breaks_ascend_and_end_the_paragraph(
        words in prop::collection::vec(5.0f64..60.0, 1..60),
        measure in 80.0f64..300.0,
    ) {
        let items = paragraph(&words);
        let broken = break_lines(&items, &fixed(measure), &Params::default());
        prop_assert!(!broken.breaks.is_empty());
        prop_assert_eq!(broken.breaks.last().unwrap().at, items.len() - 1);
        prop_assert!(broken.breaks.windows(2).all(|w| w[0].at < w[1].at));
        let widest_word = words.iter().cloned().fold(0.0, f64::max);
        if widest_word <= measure && broken.tolerance.is_some() {
            for b in &broken.breaks {
                if b.at != items.len() - 1 {
                    prop_assert!(b.ratio >= -1.0 - 1e-9);
                }
            }
        }
    }

    /// First-fit never puts a break inside a box and never loses an item.
    #[test]
    fn first_fit_covers_the_paragraph(
        words in prop::collection::vec(5.0f64..60.0, 1..60),
        measure in 80.0f64..300.0,
    ) {
        let items = paragraph(&words);
        let breaks = first_fit(&items, &fixed(measure));
        prop_assert_eq!(breaks.last().unwrap().at, items.len() - 1);
        prop_assert!(breaks.windows(2).all(|w| w[0].at < w[1].at));
        for b in &breaks {
            let is_box = matches!(items[b.at], Item::Box { .. });
            prop_assert!(!is_box);
        }
    }
}
