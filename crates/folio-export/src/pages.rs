// Which pages a PDF has, and how they are laid on its sheets
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The Sharing PDF can be the whole book or part of it, with the cover as the front alone, the
//! front and back together or left out, and each page on its own sheet or the two pages of a spread
//! side by side. Every other target is the whole book as single pages, because that is what its
//! destination wants (D-71).

use crate::outline::Origin;
use crate::pdf::PdfTarget;

/// How much of the cover the file has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoverPages {
    /// The front cover, as the first page.
    #[default]
    Front,
    /// The whole cover sheet: back, spine and front on one wide first page.
    FrontAndBack,
    /// The front cover first and the back cover last, each a page of the trim size.
    FrontAndBackEnds,
    /// No cover.
    None,
}

/// What a sheet of the file holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sheets {
    /// One page.
    #[default]
    Pages,
    /// The pages of a spread side by side; the cover and a spread of one page are alone.
    Spreads,
}

/// The pages of the book a PDF has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PageChoice {
    /// The first and last page wanted, counted from 1 over the pages the file would have, both
    /// included. `None` is every page.
    pub range: Option<(usize, usize)>,
    pub cover: CoverPages,
    pub sheets: Sheets,
}

impl PageChoice {
    /// What `target` allows: only the Sharing PDF is anything but the whole book as single pages.
    pub fn for_target(&self, target: PdfTarget) -> PageChoice {
        match target {
            PdfTarget::Sharing => *self,
            _ => PageChoice::default(),
        }
    }
}

/// The indexes of the pages `range` keeps out of `count`, counting pages from 1.
pub fn kept(count: usize, range: Option<(usize, usize)>) -> std::ops::Range<usize> {
    match range {
        None => 0..count,
        Some((first, last)) => {
            let first = first.max(1).min(count + 1) - 1;
            let last = last.min(count).max(first);
            first..last
        }
    }
}

/// Which pages go on which sheet: each sheet is the indexes of its pages, left to right.
pub fn sheets(origins: &[Origin], how: Sheets) -> Vec<Vec<usize>> {
    match how {
        Sheets::Pages => (0..origins.len()).map(|i| vec![i]).collect(),
        Sheets::Spreads => {
            let mut out: Vec<Vec<usize>> = Vec::new();
            for (at, origin) in origins.iter().enumerate() {
                match out.last_mut() {
                    Some(last) if origins[last[0]].spread == origin.spread && last.len() < 2 => {
                        last.push(at)
                    }
                    _ => out.push(vec![at]),
                }
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin(spread: &str, index: usize) -> Origin {
        Origin {
            spread: spread.into(),
            index,
            label: String::new(),
            section: String::new(),
        }
    }

    #[test]
    fn a_range_is_counted_from_one_and_kept_inside_the_book() {
        assert_eq!(kept(10, None), 0..10);
        assert_eq!(kept(10, Some((3, 5))), 2..5);
        assert_eq!(kept(10, Some((8, 99))), 7..10);
        assert_eq!(kept(10, Some((0, 2))), 0..2);
        assert!(kept(10, Some((20, 30))).is_empty());
        assert!(kept(10, Some((5, 3))).is_empty());
    }

    #[test]
    fn spreads_put_the_two_pages_of_a_spread_on_one_sheet() {
        let pages = [
            origin("cover", 0),
            origin("a", 0),
            origin("a", 1),
            origin("b", 0),
            origin("c", 0),
            origin("c", 1),
        ];
        assert_eq!(
            sheets(&pages, Sheets::Spreads),
            vec![vec![0], vec![1, 2], vec![3], vec![4, 5]]
        );
        assert_eq!(sheets(&pages, Sheets::Pages).len(), 6);
    }
}
