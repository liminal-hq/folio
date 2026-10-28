// A snapshot of the parts of a book an intent touches, to undo and redo with
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{
    BarcodeChoice, Book, BookFont, Details, Flow, FlowId, Grid, Margins, Master, MasterId,
    PageNumbers, Paper, Proofing, Pt, Size, Spread, SpreadId, Styles,
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
    pub styles: Styles,
    pub paper: Paper,
    pub barcode: BarcodeChoice,
    pub fonts: Vec<BookFont>,
    pub keywords: Vec<String>,
    pub preflight_ignore: Vec<String>,
    /// The flows a step touched, each as it was (or `None` where it did not exist).
    pub flows: Vec<(FlowId, Option<Flow>)>,
    pub spreads: Vec<(SpreadId, Option<Spread>)>,
}

impl Delta {
    /// Captures `touched` and the book-wide parts.
    pub fn capture(book: &Book, touched: &[SpreadId], flows: &[FlowId]) -> Self {
        Self {
            order: book.spreads.iter().map(|s| s.id.clone()).collect(),
            geometry: Geometry::of(book),
            masters: Masters::of(book),
            proofing: book.proofing.clone(),
            language: book.language.clone(),
            details: Details::of(book),
            styles: book.styles.clone(),
            paper: book.paper,
            barcode: book.barcode.clone(),
            fonts: book.fonts.clone(),
            keywords: book.metadata.keywords.clone(),
            preflight_ignore: book.preflight_ignore.clone(),
            flows: flows
                .iter()
                .map(|id| (id.clone(), book.flows.iter().find(|f| &f.id == id).cloned()))
                .collect(),
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

    /// The state before two steps in a row, given the state before the first (`self`) and the state
    /// before the second (`later`): the first's, and what only the second touched, which the first
    /// left as it was.
    pub fn before_both(&self, later: &Delta) -> Delta {
        let mut both = self.clone();
        for (id, spread) in &later.spreads {
            if !both.spreads.iter().any(|(i, _)| i == id) {
                both.spreads.push((id.clone(), spread.clone()));
            }
        }
        for (id, flow) in &later.flows {
            if !both.flows.iter().any(|(i, _)| i == id) {
                both.flows.push((id.clone(), flow.clone()));
            }
        }
        both
    }

    /// The state after two steps in a row, given the state after the first (`self`) and after the
    /// second (`later`): the second's, and what only the first touched, which the second left alone.
    pub fn after_both(&self, later: &Delta) -> Delta {
        let mut both = later.clone();
        for (id, spread) in &self.spreads {
            if !both.spreads.iter().any(|(i, _)| i == id) {
                both.spreads.push((id.clone(), spread.clone()));
            }
        }
        for (id, flow) in &self.flows {
            if !both.flows.iter().any(|(i, _)| i == id) {
                both.flows.push((id.clone(), flow.clone()));
            }
        }
        both
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
        let was_keywords = book.metadata.keywords.clone();
        let was_styles = book.styles.clone();
        let was_paper = book.paper;
        let was_barcode = book.barcode.clone();
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
        if was_barcode != self.barcode {
            book.barcode = self.barcode.clone();
            ops.push(PatchOp::SetBarcode {
                choice: self.barcode.clone(),
            });
        }
        if was_keywords != self.keywords {
            book.metadata.keywords = self.keywords.clone();
            ops.push(PatchOp::SetKeywords {
                keywords: self.keywords.clone(),
            });
        }
        if was_paper != self.paper {
            book.paper = self.paper;
            ops.push(PatchOp::SetPaper { paper: self.paper });
        }
        if was_styles != self.styles {
            book.styles = self.styles.clone();
            ops.push(PatchOp::SetStyles {
                styles: self.styles.clone(),
            });
        }
        if was_details != self.details {
            self.details.apply(book);
            ops.push(PatchOp::SetDetails {
                details: self.details.clone(),
            });
        }
        for (id, flow) in &self.flows {
            let slot = book.flows.iter().position(|f| &f.id == id);
            match (flow, slot) {
                (Some(flow), Some(at)) if book.flows[at] == *flow => {}
                (Some(flow), Some(at)) => {
                    book.flows[at] = flow.clone();
                    ops.push(PatchOp::SetFlow { flow: flow.clone() });
                }
                (Some(flow), None) => {
                    book.flows.push(flow.clone());
                    ops.push(PatchOp::SetFlow { flow: flow.clone() });
                }
                (None, Some(at)) => {
                    book.flows.remove(at);
                    ops.push(PatchOp::RemoveFlow { id: id.clone() });
                }
                (None, None) => {}
            }
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
