// Intents: the changes the UI asks the core to make
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::background::Background;
use crate::book::{BookFont, Details, Grid, PageNumbers};
use crate::colour::{BookSwatch, Colour};
use crate::cover::{BarcodeChoice, Paper};
use crate::flow::StoryPara;
use crate::frame::{Fit, Frame, Shape, StyleOverrides, TableLook};
use crate::ids::{FlowId, FrameId, MasterId, PageId, SpreadId, StyleId, SwatchId};
use crate::runs::RunChange;
use crate::sections::Section;
use crate::spread::Axis;
use crate::units::{Margins, Pt, Rect, Size};

/// A frame and the box it is to have.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct FrameMove {
    pub frame: FrameId,
    #[serde(rename = "box")]
    pub rect: Rect,
}

/// Where a frame or a guide lives: on a spread, or on a master page that every page using it shows.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum Surface {
    Spread(SpreadId),
    Master(MasterId),
}

/// A fill or an outline colour to set: a colour, or none at all.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Paint {
    None,
    Colour { colour: Colour },
}

/// What a background is set on.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BackgroundTarget {
    /// One page of a spread.
    Page { spread: SpreadId, page: PageId },
    /// Both pages of a spread, through the gutter. A single page has no such thing.
    Span { spread: SpreadId },
    /// Every page after the cover that has none of its own, by way of the masters they use.
    EveryPage,
}

/// A change to a table's cells, rows, columns or look.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TableEdit {
    /// Puts the words of one cell; empty words empty the cell.
    SetCell {
        row: usize,
        col: usize,
        text: String,
    },
    /// Adds an empty row so it is row `at` (the end is one past the last). The table grows by a row's
    /// height, so the other rows keep theirs.
    AddRow {
        at: usize,
    },
    /// Takes row `at` out and the table shrinks by its height. A table keeps at least one row.
    RemoveRow {
        at: usize,
    },
    /// Adds an empty column so it is column `at`, taking an equal share and leaving the others
    /// proportionally narrower.
    AddColumn {
        at: usize,
    },
    /// Takes column `at` out, the others widening to fill. A table keeps at least one column.
    RemoveColumn {
        at: usize,
    },
    /// How many rows from the top are headers.
    SetHead {
        head: u8,
    },
    SetLook {
        look: TableLook,
    },
}

// ── Intents (UI → core) ──
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(tag = "type")]
// `AddFrame` carries a whole frame and the rest are small. Intents are made one at a time and
// passed along, never kept in bulk, so the extra bytes cost nothing worth a `Box` in every frame.
#[allow(clippy::large_enum_variant)]
pub enum Intent {
    MoveFrame {
        on: Surface,
        frame: FrameId,
        #[serde(rename = "box")]
        rect: Rect,
    },
    /// Moves or resizes several frames together, as one change that merges with the next of the
    /// same frames within the coalescing window, so a drag or a nudge of a selection is one undo step.
    MoveFrames {
        on: Surface,
        moves: Vec<FrameMove>,
    },
    /// Puts a copy of a frame on the same spread, a little down and to the right, in front.
    DuplicateFrame {
        on: Surface,
        frame: FrameId,
    },
    /// Locks a frame against moving and deleting, or lets it go again.
    LockFrame {
        on: Surface,
        frame: FrameId,
        locked: bool,
    },
    /// Puts a picture in a picture frame (an asset path from the package) and says how it sits in
    /// the frame, including which point of it stays in view when it is cropped (as 0–1 across and
    /// down). A part left out stays as it was, and a drag of the focus point is one undo step.
    SetImage {
        frame: FrameId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        asset: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fit: Option<Fit>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        focus: Option<(f64, f64)>,
    },
    /// Changes how a shape looks: its kind, its fill and outline, how round its corners are and which
    /// way a line runs. A part left out stays as it was, and a trying-out of colours or widths on one
    /// shape is one undo step.
    SetShape {
        frame: FrameId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<Shape>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<Paint>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<Paint>,
        #[serde(
            default,
            rename = "strokeWidth",
            skip_serializing_if = "Option::is_none"
        )]
        stroke_width: Option<Pt>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        radius: Option<Pt>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        flip: Option<bool>,
    },
    /// Turns a frame to `degrees` clockwise about its middle (0 is upright). Any angle is kept in 0
    /// to 360.
    RotateFrame {
        on: Surface,
        frame: FrameId,
        degrees: f64,
    },
    /// Sets the language the book is written in, as a BCP 47 code such as `en-CA` or `fr`. It
    /// decides which dictionary checks the spelling, and how words are hyphenated and shaped.
    SetLanguage {
        language: String,
    },
    /// Sets the fonts the book keeps, each a file already in the package. The books plugin adds the
    /// files and sends this; it is one undo step.
    SetFonts {
        fonts: Vec<BookFont>,
    },
    /// Sets the words of one paragraph of a flow's story, keeping the look of the words left alone.
    /// Typing in one paragraph is one undo step.
    SetParagraphText {
        flow: FlowId,
        para: usize,
        text: String,
    },
    /// Splits a paragraph of a flow's story at character `at`: Enter in the middle of its words. The
    /// new paragraph after it has the same style, and both keep the look of their words.
    SplitParagraph {
        flow: FlowId,
        para: usize,
        at: usize,
    },
    /// Joins a paragraph of a flow's story to the one after it: Backspace at the start of that one.
    JoinParagraphs {
        flow: FlowId,
        para: usize,
    },
    /// Replaces `remove` paragraphs of a flow's story from place `at` with `insert`: Enter, Backspace
    /// across paragraphs, paste and deleting a stretch of paragraphs are all this. One undo step.
    SpliceStory {
        flow: FlowId,
        at: usize,
        remove: usize,
        insert: Vec<StoryPara>,
    },
    /// Sets the paragraph style of paragraphs `from..=to` of a flow's story; none takes it away.
    StyleParagraphs {
        flow: FlowId,
        from: usize,
        to: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        style: Option<StyleId>,
    },
    /// Changes the look of characters `start..end` of one paragraph of a flow's story.
    StyleStoryRange {
        flow: FlowId,
        para: usize,
        start: u32,
        end: u32,
        change: RunChange,
    },
    /// Adds `count` frames like the last one of a flow, each after the one before it, on the next
    /// pages: the other page of the same spread when it is free, new spreads otherwise. One undo
    /// step. It is what a story that has outgrown its frames asks for.
    AddFlowPages {
        flow: FlowId,
        count: usize,
    },
    /// Takes the frames of a flow after the first `keep` out of it and off the pages, and the spreads
    /// they leave with nothing on them: what a story that has shrunk no longer needs. One undo step.
    TrimFlowPages {
        flow: FlowId,
        keep: usize,
    },
    /// Chains the text frame `frame` after the text frame `to`, so the words that run out of `to`
    /// continue in it. `to` starts a flow when it was not in one, taking its words as the story;
    /// the words `frame` held are added to the end of the story.
    LinkFrame {
        frame: FrameId,
        to: FrameId,
    },
    /// Takes a text frame out of its flow. The frames after it stay chained; a frame that was the
    /// only one takes the story back as its own words.
    UnlinkFrame {
        frame: FrameId,
    },
    /// Sets who makes the barcode on the paperback cover. One undo step.
    SetBarcode {
        choice: BarcodeChoice,
    },
    /// Sets the paper a printed copy is on. One undo step.
    SetPaper {
        paper: Paper,
    },
    /// Adds a swatch to the book, or replaces the one with the same id. A swatch with ink shows as
    /// that ink on a screen, so its display colour is worked out from it. One undo step.
    SetSwatch {
        swatch: BookSwatch,
    },
    /// Takes a swatch out of the book. Everything that referred to it keeps the colour it showed
    /// as, in the same undo step.
    RemoveSwatch {
        id: SwatchId,
    },
    /// Sets the title, subtitle, authors, description and ISBNs together. One undo step.
    SetDetails {
        details: Details,
    },
    /// Sets the preflight issues to leave alone, each by its key. One undo step.
    SetPreflightIgnore {
        keys: Vec<String>,
    },
    /// Closes the last undo step, so the next change on the same frame starts a new one instead of
    /// joining it. It changes nothing in the book. Words typed in an editor send it before each pause
    /// or paste is kept, so undo takes back one stretch of typing at a time.
    EndUndoStep,
    /// Tells spelling to leave a word alone in this book, or to check it again.
    IgnoreWord {
        word: String,
    },
    UnignoreWord {
        word: String,
    },
    /// Starts a section at a spread, or changes the one it starts; `None` takes it away so the
    /// spread carries on the section before. The cover cannot start one.
    SetSection {
        spread: SpreadId,
        section: Option<Section>,
    },
    /// Puts page numbers on every page after the cover, or takes them off.
    SetPageNumbers {
        position: PageNumbers,
    },
    /// Makes master page A (and B when the page numbers are outside) if the book has none, and gives
    /// every page after the cover without a master the one its side uses, so a master can be edited.
    EnsureMasters,
    /// Hides a master page's frame on one page, or shows it again.
    HideMasterItem {
        spread: SpreadId,
        page: PageId,
        frame: FrameId,
        hide: bool,
    },
    /// Hides a master page's frame on one page and puts an editable copy of it there, so the page
    /// can change it without changing the master. The copy is the person's own.
    CopyMasterItem {
        spread: SpreadId,
        page: PageId,
        frame: FrameId,
    },
    /// Sets, or with `None` clears, a background (D-23). "Every page" makes the masters if the book
    /// has none, and a page's own background still wins over its master's.
    SetBackground {
        target: BackgroundTarget,
        bg: Option<Background>,
    },
    /// Puts a ruler guide on a spread, or on a master page so it shows on every page using it.
    AddGuide {
        on: Surface,
        axis: Axis,
        pos: Pt,
    },
    /// Moves a ruler guide, named by its place in the spread's list. A locked guide stays put.
    MoveGuide {
        on: Surface,
        index: usize,
        pos: Pt,
    },
    /// Locks a ruler guide where it is, or frees it.
    LockGuide {
        on: Surface,
        index: usize,
        locked: bool,
    },
    /// Takes a ruler guide off its spread. A locked guide stays.
    RemoveGuide {
        on: Surface,
        index: usize,
    },
    /// Moves a ruler guide from a spread onto the masters its pages use, so it shows on every page
    /// using them. A horizontal guide goes on each of them; a vertical one on the master of the page
    /// it is over (as a position on that page), and mirrored onto the facing master when there is one.
    PromoteGuide {
        spread: SpreadId,
        index: usize,
    },
    /// Moves a guide from a master back onto one spread that uses it, so it shows only there.
    DemoteGuide {
        master: MasterId,
        index: usize,
        spread: SpreadId,
    },
    /// Takes the unlocked guides off one spread, or off every spread when none is named.
    ClearGuides {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spread: Option<SpreadId>,
    },
    SetText {
        frame: FrameId,
        text: String,
    },
    /// Changes the look of the characters `start..end` of a frame's words (counted in characters),
    /// splitting the words into runs where it begins and ends. One undo step.
    StyleRange {
        frame: FrameId,
        start: u32,
        end: u32,
        change: RunChange,
    },
    /// Changes a table: a cell's words, its rows and columns, header rows or look. Typing in one
    /// cell is one undo step.
    EditTable {
        frame: FrameId,
        edit: TableEdit,
    },
    SetStyle {
        frame: FrameId,
        style: StyleId,
    },
    OverrideStyle {
        frame: FrameId,
        overrides: StyleOverrides,
    },
    AddFrame {
        on: Surface,
        frame: Frame,
    },
    DeleteFrame {
        on: Surface,
        frame: FrameId,
    },
    Reorder {
        on: Surface,
        frame: FrameId,
        to: ReorderTo,
    },
    AddSpread {
        after: SpreadId,
        duplicate: bool,
    },
    DeleteSpread {
        spread: SpreadId,
    },
    MoveSpread {
        spread: SpreadId,
        to: u32,
    },
    SetBookGeometry {
        trim: Option<Size>,
        margins: Option<Margins>,
        bleed: Option<Pt>,
        grid: Option<Grid>,
    },
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum ReorderTo {
    Front,
    Back,
    Index(u32),
}
