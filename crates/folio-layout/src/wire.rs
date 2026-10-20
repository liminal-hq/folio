// What a laid-out spread looks like to the UI: lines of positioned text per frame, and the checks
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{Colour, FrameId, SpreadId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A word, or a piece of one, drawn in one font. `x` is from the line's left edge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct LayoutRun {
    pub text: String,
    pub x: f64,
    pub width: f64,
    /// The family actually used, which may differ from the one asked for.
    pub family: String,
    pub size: f64,
    pub weight: u16,
    pub italic: bool,
    /// Extra space after every glyph, in ems.
    pub tracking: f64,
    /// The colour of the words, when the style or the run sets one; the page's ink otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colour: Option<Colour>,
    /// A hyphen drawn at the end of a line; it is not part of the text.
    pub hyphen: bool,
    /// The misspelt parts of `text`, each as where it starts and ends in UTF-16 code units.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub miss: Vec<(u32, u32)>,
    /// The key of the font file the run was shaped with (`Font::face_key`). Not sent to the page:
    /// the page draws text itself, and only the exporters want this.
    #[serde(skip)]
    #[ts(skip)]
    pub face_key: String,
    /// The shaped glyphs of the run, in order. Not sent to the page.
    #[serde(skip)]
    #[ts(skip)]
    pub glyphs: Vec<SceneGlyph>,
}

/// One shaped glyph of a run: which glyph of the font, and how far to move for it, in points.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SceneGlyph {
    pub id: u16,
    /// Byte offset into the run's text of the first character the glyph stands for.
    pub cluster: u32,
    pub advance: f64,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct LayoutLine {
    /// Left edge of the line and the top of its box, from the frame's top-left, in points.
    pub x: f64,
    pub top: f64,
    pub baseline: f64,
    pub width: f64,
    pub height: f64,
    pub runs: Vec<LayoutRun>,
}

/// A rectangle in points, from a frame's top-left.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct FrameLayout {
    pub frame: FrameId,
    pub lines: Vec<LayoutLine>,
    /// The frame's text carries on past its last line, in a frame with no room.
    pub overset: bool,
    /// For a frame that comes from a master page: where its top-left sits on the spread, since the
    /// spread has no such frame to say. `frame` is then the master frame's id and the page's, joined
    /// by `@`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
    /// For a shape: the room its words have, from its top-left, where an editor goes. The lines
    /// are already placed inside it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_box: Option<LayoutRect>,
    /// For a shape: how its style sets the words, so an editor sets them the same way.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_align: Option<folio_model::Align>,
}

/// A point on a spread, in points from the top-left of the first page's trim.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Origin {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum PreflightKind {
    /// Text runs on past the last frame of its chain.
    OversetText,
    /// A text frame reaches outside the safe margin.
    TextOutsideMargin,
    /// A picture touches the trim edge but stops short of the bleed.
    PictureShortOfBleed,
    /// A preserved poem line was too long and wrapped.
    PoemLineTooLong,
    /// A picture frame or background names a picture that is not in the book.
    PictureMissing,
    /// A picture is drawn at under 300 pixels per inch: a warning.
    LowResolution,
    /// A picture is drawn at under 150 pixels per inch: an error.
    VeryLowResolution,
    /// A page has a background but the book has no bleed, so a trimmed edge may show paper.
    BleedNeeded,
    /// A text frame asks for a font the book does not have and this computer lacks, so it is set
    /// in a stand-in and its lines will break differently from where the book was made. The item's
    /// `family` names it.
    FontMissing,
    /// A picture frame has no picture yet.
    PicturePlaceholder,
    /// A picture has no description for screen readers.
    AltMissing,
    /// Words use characters no font Folio has covers, so they are drawn as empty boxes. The item's
    /// `chars` names them.
    MissingGlyphs,
}

/// How much an issue matters: an error waits on the person, a warning is worth a look and a note
/// is only for information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl PreflightKind {
    /// How much an issue of this kind matters.
    pub fn severity(self) -> Severity {
        match self {
            Self::OversetText
            | Self::PictureMissing
            | Self::VeryLowResolution
            | Self::PicturePlaceholder => Severity::Error,
            Self::AltMissing => Severity::Note,
            Self::MissingGlyphs => Severity::Warning,
            Self::TextOutsideMargin
            | Self::PictureShortOfBleed
            | Self::PoemLineTooLong
            | Self::LowResolution
            | Self::BleedNeeded
            | Self::FontMissing => Severity::Warning,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct PreflightItem {
    pub kind: PreflightKind,
    pub severity: Severity,
    pub spread: SpreadId,
    /// The frame at fault; none when it is a page's background.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<FrameId>,
    /// The page at fault, for a background.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<folio_model::PageId>,
    /// The typeface at fault, for a missing font.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// The characters at fault, for words no font covers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chars: Option<String>,
    /// What names this issue (see `compute_key`); set when a whole book is checked.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub key: String,
    /// The person said to leave this one alone.
    #[serde(default, skip_serializing_if = "is_false")]
    pub ignored: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl PreflightItem {
    /// What names this issue, so a person can say to leave it alone and it stays left alone: the
    /// kind, where, and what it is about.
    pub fn compute_key(&self) -> String {
        let at = self
            .frame
            .as_ref()
            .map(|f| f.0.as_str())
            .or(self.page.as_ref().map(|p| p.0.as_str()))
            .unwrap_or("");
        format!(
            "{:?}:{}:{}:{}{}",
            self.kind,
            self.spread.0,
            at,
            self.family.as_deref().unwrap_or(""),
            self.chars.as_deref().unwrap_or("")
        )
    }

    /// A problem with one frame of `spread`.
    pub fn of_frame(kind: PreflightKind, spread: &SpreadId, frame: &FrameId) -> Self {
        Self {
            kind,
            severity: kind.severity(),
            spread: spread.clone(),
            frame: Some(frame.clone()),
            page: None,
            family: None,
            chars: None,
            key: String::new(),
            ignored: false,
        }
    }

    /// A problem with the background of one page of `spread`.
    pub fn of_page(kind: PreflightKind, spread: &SpreadId, page: &folio_model::PageId) -> Self {
        Self {
            kind,
            severity: kind.severity(),
            spread: spread.clone(),
            frame: None,
            page: Some(page.clone()),
            family: None,
            chars: None,
            key: String::new(),
            ignored: false,
        }
    }

    /// A frame of `spread` whose words use `chars`, which no font covers.
    pub fn missing_glyphs(spread: &SpreadId, frame: &FrameId, chars: &str) -> Self {
        Self {
            chars: Some(chars.to_owned()),
            ..Self::of_frame(PreflightKind::MissingGlyphs, spread, frame)
        }
    }

    /// A frame of `spread` set in a stand-in because `family` is missing.
    pub fn font_missing(spread: &SpreadId, frame: &FrameId, family: &str) -> Self {
        Self {
            family: Some(family.to_owned()),
            ..Self::of_frame(PreflightKind::FontMissing, spread, frame)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct SpreadLayout {
    pub spread: SpreadId,
    pub frames: Vec<FrameLayout>,
    pub preflight: Vec<PreflightItem>,
}
