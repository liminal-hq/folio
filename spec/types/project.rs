//! folio-model — the Folio document model (source of truth).
//! TypeScript bindings are generated with ts-rs into packages/protocol (mirrored by spec/types/project.ts).
//! All lengths are points (1/72 in). (c) Liminal HQ · Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize, TS)]
#[serde(transparent)]
pub struct Pt(pub f64);
impl Pt {
    pub const PER_INCH: f64 = 72.0;
    pub fn from_in(v: f64) -> Self { Pt(v * Self::PER_INCH) }
    pub fn from_mm(v: f64) -> Self { Pt(v * Self::PER_INCH / 25.4) }
    pub fn to_in(self) -> f64 { self.0 / Self::PER_INCH }
}

macro_rules! id { ($($n:ident),*) => { $(
    #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
    #[serde(transparent)] pub struct $n(pub String);
)* } }
id!(BookId, SpreadId, PageId, FrameId, FlowId, StyleId, MasterId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum BookKind { Picture, Novel, Textbook, Illustrated, Poetry, Memoir }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum LayoutMode { Fixed, Flowing }

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
pub struct Size { pub w: Pt, pub h: Pt }
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
pub struct Rect { pub x: Pt, pub y: Pt, pub w: Pt, pub h: Pt }
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
pub struct Margins { pub top: Pt, pub bottom: Pt, pub inside: Pt, pub outside: Pt }

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub schema: u32,
    pub id: BookId,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub subtitle: Option<String>,
    pub authors: Vec<Contributor>,
    pub language: String,
    pub kind: BookKind,
    pub layout: LayoutMode,
    pub trim: Size,
    pub bleed: Pt,
    pub margins: Margins,
    #[serde(default)] pub adjust_gutter_for_page_count: bool,
    pub grid: Grid,
    pub spreads: Vec<Spread>,
    #[serde(default)] pub flows: Vec<Flow>,
    #[serde(default)] pub masters: Vec<MasterId>,
    pub metadata: BookMetadata,
    /// Unknown data from imports / newer versions, preserved verbatim.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(type = "Record<string, unknown>")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Contributor { pub name: String, pub role: ContributorRole }
#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ContributorRole { Author, Illustrator, Editor, Translator }

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Grid { pub columns: u8, pub gutter: Pt, pub baseline: Option<Baseline> }
#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
pub struct Baseline { pub increment: Pt, pub offset: Pt }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum SpreadKind { Cover, Single, Spread, Opener, Closer }

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Spread {
    pub id: SpreadId,
    pub kind: SpreadKind,
    pub pages: Vec<Page>,
    /// z-order = vector order
    pub frames: Vec<Frame>,
    #[serde(default)] pub guides: Vec<Guide>,
    #[serde(default)] pub master_overrides: Vec<MasterOverride>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Page { pub id: PageId, pub master: Option<MasterId> }
#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
pub struct Guide { pub axis: Axis, pub pos: Pt, pub locked: bool }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Axis { X, Y }
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MasterOverride { pub page_id: PageId, pub hide: Vec<FrameId> }

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Frame {
    Text(TextFrame),
    Image(ImageFrame),
    Shape(ShapeFrame),
    Group { #[serde(flatten)] base: FrameBase, children: Vec<Frame> },
}
impl Frame {
    pub fn base(&self) -> &FrameBase {
        match self { Frame::Text(f) => &f.base, Frame::Image(f) => &f.base, Frame::Shape(f) => &f.base, Frame::Group { base, .. } => base }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FrameBase {
    pub id: FrameId,
    #[serde(rename = "box")] pub rect: Rect,
    #[serde(default)] pub rotation: f64,
    #[serde(default)] pub locked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub from_master: Option<MasterId>,
    /// Alt text for pictures; accessible name for others.
    #[serde(default, skip_serializing_if = "Option::is_none")] pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TextFrame {
    #[serde(flatten)] pub base: FrameBase,
    pub style: StyleId,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub content: Option<InlineContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub flow: Option<FlowSlot>,
    #[serde(default = "one")] pub columns: u8,
    #[serde(default)] pub baseline_lock: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub overrides: Option<StyleOverrides>,
}
fn one() -> u8 { 1 }

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct InlineContent { pub inline: Vec<Run> }
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct Run { pub text: String, #[serde(default)] pub style: Option<StyleId>, #[serde(default)] pub bold: bool, #[serde(default)] pub italic: bool }
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct FlowSlot { pub id: FlowId, pub index: u32 }

#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
pub struct StyleOverrides { pub size: Option<Pt>, pub align: Option<Align>, pub bold: Option<bool>, pub italic: Option<bool> }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Align { Left, Center, Right, Justify }

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ImageFrame {
    #[serde(flatten)] pub base: FrameBase,
    pub image: ImageRef,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub placeholder_label: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct ImageRef { pub asset: Option<String>, pub fit: Fit, pub focus: (f64, f64), pub crop: Option<Rect> }
#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Fit { Fill, Fit, Stretch, Manual }

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct ShapeFrame { #[serde(flatten)] pub base: FrameBase, pub shape: Shape, pub fill: Option<String>, pub style: Option<StyleId> }
#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Shape { Rect, Rounded, Ellipse }

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Flow { pub id: FlowId, pub source: String, pub frames: Vec<FrameId>, #[serde(default = "yes")] pub auto_add_pages: bool }
fn yes() -> bool { true }

#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
pub struct BookMetadata { pub isbn: Isbns, pub description: String, #[serde(default)] pub keywords: Vec<String> }
#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
pub struct Isbns { pub paperback: Option<String>, pub hardcover: Option<String>, pub ebook: Option<String> }

// ── Export maths ──
#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Paper { White, Cream, Colour }

/// KDP paperback spine factor, inches per page. ⚠ Verify against KDP's current cover calculator.
pub fn spine_factor(p: Paper) -> f64 { match p { Paper::White => 0.002252, Paper::Cream => 0.0025, Paper::Colour => 0.002347 } }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
pub struct CoverMaths { pub spine: Pt, pub edge: Pt, pub width: Pt, pub height: Pt, pub spine_text_allowed: bool, pub barcode: Rect }

pub fn paperback_cover(trim: Size, pages: u32, paper: Paper) -> CoverMaths {
    let spine = Pt::from_in(pages as f64 * spine_factor(paper));
    let edge = Pt::from_in(0.125);
    let width = Pt(edge.0 * 2.0 + trim.w.0 * 2.0 + spine.0);
    let height = Pt(edge.0 * 2.0 + trim.h.0);
    // 2 × 1.2 in barcode box, 0.25 in in from the back cover's bottom-right trim corner.
    let barcode = Rect { x: Pt(edge.0 + trim.w.0 - Pt::from_in(2.25).0), y: Pt(edge.0 + trim.h.0 - Pt::from_in(1.45).0), w: Pt::from_in(2.0), h: Pt::from_in(1.2) };
    CoverMaths { spine, edge, width, height, spine_text_allowed: pages >= 79, barcode }
}

// ── Snapping (shared maths; mirrors Spindle SceneCanvas, extended for books) ──
pub const SNAP_THRESHOLD_PX: f64 = 8.0;

/// Returns (shift, target) for the closest candidate/target pair within `threshold`.
pub fn snap_axis(candidates: &[f64], targets: &[f64], threshold: f64) -> Option<(f64, f64)> {
    let mut best: Option<(f64, f64, f64)> = None; // (dist, shift, target)
    for &c in candidates { for &t in targets {
        let d = (t - c).abs();
        if d <= threshold && best.map_or(true, |b| d < b.0) { best = Some((d, t - c, t)); }
    } }
    best.map(|(_, s, t)| (s, t))
}

// ── Intents (UI → core) ──
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
pub enum Intent {
    MoveFrame { spread: SpreadId, frame: FrameId, #[serde(rename = "box")] rect: Rect },
    SetText { frame: FrameId, text: String },
    SetStyle { frame: FrameId, style: StyleId },
    OverrideStyle { frame: FrameId, overrides: StyleOverrides },
    AddFrame { spread: SpreadId, frame: Frame },
    DeleteFrame { spread: SpreadId, frame: FrameId },
    Reorder { spread: SpreadId, frame: FrameId, to: ReorderTo },
    AddSpread { after: SpreadId, duplicate: bool },
    DeleteSpread { spread: SpreadId },
    MoveSpread { spread: SpreadId, to: u32 },
    SetBookGeometry { trim: Option<Size>, margins: Option<Margins>, bleed: Option<Pt>, grid: Option<Grid> },
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ReorderTo { Front, Back, Index(u32) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn otter_cover_maths() {
        let c = paperback_cover(Size { w: Pt::from_in(8.5), h: Pt::from_in(8.5) }, 32, Paper::White);
        assert!((c.spine.to_in() - 0.072064).abs() < 1e-6);
        assert!((c.width.to_in() - 17.322064).abs() < 1e-6);
        assert!(!c.spine_text_allowed);
    }
    #[test]
    fn snaps_to_nearest() {
        assert_eq!(snap_axis(&[10.0, 15.0], &[0.0, 16.0], 2.0), Some((1.0, 16.0)));
        assert_eq!(snap_axis(&[10.0], &[20.0], 2.0), None);
    }
}
