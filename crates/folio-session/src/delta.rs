// A snapshot of the parts of a book an intent touches, to undo and redo with
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{
    Book, BookFont, Details, Grid, Margins, Master, MasterId, PageNumbers, Proofing, Pt, Size,
    Spread, SpreadId,
};

use crate::patch::PatchOp;

/// The page geometry of a book.
#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    pub trim: Size,
    pub margins: Margins,
    pub bleed: Pt,
    pub grid: Grid,
}

impl Geometry {
    pub fn of(book: &Book) -> Self {
        Self {
            trim: book.trim,
            margins: book.margins,
            bleed: book.bleed,
            grid: book.grid.clone(),
        }
    }

    fn apply(&self, book: &mut Book) {
        book.trim = self.trim;
        book.margins = self.margins;
        book.bleed = self.bleed;
        book.grid = self.grid.clone();
    }

    fn op(&self) -> PatchOp {
        PatchOp::SetGeometry {
            trim: self.trim,
            margins: self.margins,
            bleed: self.bleed,
            grid: self.grid.clone(),
        }
    }
}

/// The master pages of a book.
#[derive(Debug, Clone, PartialEq)]
pub struct Masters {
    pub pages: Vec<Master>,
    pub ids: Vec<MasterId>,
    pub numbers: PageNumbers,
}

impl Masters {
    pub fn of(book: &Book) -> Self {
        Self {
            pages: book.master_pages.clone(),
            ids: book.masters.clone(),
            numbers: book.page_numbers,
        }
    }

    fn apply(&self, book: &mut Book) {
        book.master_pages = self.pages.clone();
        book.masters = self.ids.clone();
        book.page_numbers = self.numbers;
    }

    pub fn op(&self) -> PatchOp {
        PatchOp::SetMasters {
            masters: self.pages.clone(),
            ids: self.ids.clone(),
            numbers: self.numbers,
        }
    }
}

/// How the touched part of a book was at one moment: the order of every spread, the geometry, and
/// each touched spread (or `None` where it did not exist).
#[derive(Debug, Clone)]
pub struct Delta {
    pub order: Vec<SpreadId>,
    pub geometry: Geometry,
    pub masters: Masters,
    pub proofing: Proofing,
    pub language: String,
    pub details: Details,
    pub fonts: Vec<BookFont>,
    pub preflight_ignore: Vec<String>,
    pub spreads: Vec<(SpreadId, Option<Spread>)>,
}

impl Delta {
    /// Captures `touched` and the book-wide parts.
    pub fn capture(book: &Book, touched: &[SpreadId]) -> Self {
        Self {
            order: book.spreads.iter().map(|s| s.id.clone()).collect(),
            geometry: Geometry::of(book),
            masters: Masters::of(book),
            proofing: book.proofing.clone(),
            language: book.language.clone(),
            details: Details::of(book),
            fonts: book.fonts.clone(),
            preflight_ignore: book.preflight_ignore.clone(),
            spreads: touched
                .iter()
                .map(|id| {
                    (
                        id.clone(),
                        book.spreads.iter().find(|s| &s.id == id).cloned(),
                    )
                })
                .collect(),
        }
    }

    /// Puts the book back as this snapshot has it, and says what that changed.
    pub fn restore(&self, book: &mut Book) -> Vec<PatchOp> {
        let mut ops = Vec::new();
        let was_order: Vec<SpreadId> = book.spreads.iter().map(|s| s.id.clone()).collect();
        let was_geometry = Geometry::of(book);
        let was_masters = Masters::of(book);
        let was_proofing = book.proofing.clone();
        let was_language = book.language.clone();
        let was_details = Details::of(book);
        let was_fonts = book.fonts.clone();
        let was_ignore = book.preflight_ignore.clone();

        for (id, spread) in &self.spreads {
            match spread {
                Some(spread) => {
                    match book.spreads.iter_mut().find(|s| &s.id == id) {
                        Some(slot) => *slot = spread.clone(),
                        None => book.spreads.push(spread.clone()),
                    }
                    ops.push(PatchOp::SetSpread {
                        spread: spread.clone(),
                    });
                }
                None => {
                    book.spreads.retain(|s| &s.id != id);
                    ops.push(PatchOp::RemoveSpread { id: id.clone() });
                }
            }
        }
        // Put the spreads in the snapshot's order; any left over (there should be none) keep their place at the end.
        let rank = |id: &SpreadId| {
            self.order
                .iter()
                .position(|o| o == id)
                .unwrap_or(usize::MAX)
        };
        book.spreads.sort_by_key(|s| rank(&s.id));
        if was_order != self.order {
            ops.push(PatchOp::SetOrder {
                order: self.order.clone(),
            });
        }
        if was_geometry != self.geometry {
            self.geometry.apply(book);
            ops.push(self.geometry.op());
        }
        if was_masters != self.masters {
            self.masters.apply(book);
            ops.push(self.masters.op());
        }
        if was_language != self.language {
            book.language = self.language.clone();
            ops.push(PatchOp::SetLanguage {
                language: self.language.clone(),
            });
        }
        if was_details != self.details {
            self.details.apply(book);
            ops.push(PatchOp::SetDetails {
                details: self.details.clone(),
            });
        }
        if was_ignore != self.preflight_ignore {
            book.preflight_ignore = self.preflight_ignore.clone();
            ops.push(PatchOp::SetPreflightIgnore {
                keys: self.preflight_ignore.clone(),
            });
        }
        if was_fonts != self.fonts {
            book.fonts = self.fonts.clone();
            ops.push(PatchOp::SetFonts {
                fonts: self.fonts.clone(),
            });
        }
        if was_proofing != self.proofing {
            book.proofing = self.proofing.clone();
            ops.push(PatchOp::SetProofing {
                proofing: self.proofing.clone(),
            });
        }
        ops
    }
}
