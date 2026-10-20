// What the UI applies to its copy of the book after a change
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{
    BackNote, BarcodeChoice, BookFont, Details, Flow, FlowId, Grid, Margins, Master, MasterId,
    PageNumbers, Paper, Proofing, Pt, Size, Spread, SpreadId, Styles,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One change to the UI's copy of the book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "op", rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum PatchOp {
    /// Adds a spread, or replaces the one with the same id.
    SetSpread { spread: Spread },
    /// Removes a spread.
    RemoveSpread { id: SpreadId },
    /// The spreads, in order. Sent whenever the order changes.
    SetOrder { order: Vec<SpreadId> },
    /// The master pages, which page numbers they make, and the ids in order.
    SetMasters {
        masters: Vec<Master>,
        ids: Vec<MasterId>,
        numbers: PageNumbers,
    },
    /// The language the book is written in.
    SetLanguage { language: String },
    /// Who makes the barcode on the paperback cover.
    SetBarcode { choice: BarcodeChoice },
    /// The words on the back cover, whole.
    SetBackNotes { notes: Vec<BackNote> },
    /// The paper a printed copy is on.
    SetPaper { paper: Paper },
    /// A flow's story and frames, whole.
    SetFlow { flow: Flow },
    /// A flow that is gone.
    RemoveFlow { id: FlowId },
    /// The book's styles and swatches, whole.
    SetStyles { styles: Styles },
    /// The title, authors, description and ISBNs.
    SetDetails { details: Details },
    /// The preflight issues the book leaves alone.
    SetPreflightIgnore { keys: Vec<String> },
    /// The fonts the book keeps.
    SetFonts { fonts: Vec<BookFont> },
    /// The words spelling leaves alone in the book.
    SetProofing { proofing: Proofing },
    /// The book's page geometry.
    SetGeometry {
        trim: Size,
        margins: Margins,
        bleed: Pt,
        grid: Grid,
    },
}

/// The answer to an intent, an undo or a redo: what changed, and the revision it brings the book to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Patch {
    /// Starts at 0 for the book as opened and grows by one per change, undo and redo.
    #[ts(type = "number")]
    pub revision: u64,
    pub ops: Vec<PatchOp>,
}
