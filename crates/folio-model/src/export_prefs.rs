// Export settings: what a person chose for their PDFs, kept for a book and for themselves
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Every field is optional so each layer of settings (the built-in ones, your defaults, the book's
//! own, this export's) can leave a choice unset and let the layer below it decide. Combining the
//! layers and applying what each target locks is `folio_export::prefs::resolve`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How much to keep of a book's pictures in a PDF.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum PicturePreset {
    /// For email.
    Small,
    /// Sharp on a screen and a modest file.
    Standard,
    /// For a press: 300 ppi.
    High,
    /// Every picture exactly as it was brought in.
    Original,
}

/// How pictures that are not passed through are compressed.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum PictureCompression {
    /// Photographs as JPEG, flat art and transparency lossless.
    Auto,
    /// Every picture as JPEG, except where transparency needs lossless.
    Jpeg,
    /// Nothing lossy.
    Lossless,
}

/// What the bookmarks panel lists.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum BookmarkChoice {
    None,
    Chapters,
    Spreads,
    Pages,
}

/// How a viewer lays pages out when the file opens.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum ViewerLayoutChoice {
    Single,
    Spreads,
}

/// What each sheet of the file holds.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum SheetsChoice {
    Pages,
    Spreads,
}

/// How much of the cover the file has.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum CoverChoice {
    Front,
    FrontAndBack,
    None,
}

/// The print shop's bleed.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum BleedChoice {
    /// The book's own.
    Book,
    /// This many points.
    Custom { points: f64 },
}

/// The choices for one kind of PDF.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase", default)]
pub struct PdfPrefs {
    // Pictures.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub preset: Option<PicturePreset>,
    /// The most pixels per inch a picture keeps.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_ppi: Option<u32>,
    /// JPEG quality, 1 to 100.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub jpeg_quality: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub compression: Option<PictureCompression>,
    /// A JPEG is never touched.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub keep_jpeg: Option<bool>,
    /// Pixels outside a picture's frame are cut away.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub crop_to_frame: Option<bool>,
    // Navigation and information.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub bookmarks: Option<BookmarkChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub page_labels: Option<bool>,
    /// Links in the file; the exporter does not write any yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub links: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub viewer_layout: Option<ViewerLayoutChoice>,
    /// The file's title, when not the book's.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub authors: Option<Vec<String>>,
    /// The file's subject, when not the book's description.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub language: Option<String>,
    // Pages and layout.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub sheets: Option<SheetsChoice>,
    /// The first and last page wanted, counted from 1, both included.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub range: Option<(u32, u32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cover: Option<CoverChoice>,
    // Print extras.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub crop_marks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub bleed: Option<BleedChoice>,
    /// Room round the bleed for marks, in points.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub slug: Option<f64>,
    /// Colours and pictures as ink.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub ink: Option<bool>,
    /// The press's CMYK ICC file.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub profile: Option<String>,
}

impl PdfPrefs {
    /// Whether nothing is chosen.
    pub fn is_empty(&self) -> bool {
        self == &PdfPrefs::default()
    }

    /// These choices, with `lower`'s filling every one that is not made here.
    pub fn over(&self, lower: &PdfPrefs) -> PdfPrefs {
        macro_rules! pick {
            ($($f:ident),* $(,)?) => {
                PdfPrefs { $($f: self.$f.clone().or_else(|| lower.$f.clone())),* }
            };
        }
        pick!(
            preset,
            max_ppi,
            jpeg_quality,
            compression,
            keep_jpeg,
            crop_to_frame,
            bookmarks,
            page_labels,
            links,
            viewer_layout,
            title,
            authors,
            subject,
            keywords,
            language,
            sheets,
            range,
            cover,
            crop_marks,
            bleed,
            slug,
            ink,
            profile,
        )
    }

    /// Why these choices cannot be kept, or `None` when they can.
    pub fn check(&self) -> Result<(), String> {
        if self.jpeg_quality.is_some_and(|q| !(1..=100).contains(&q)) {
            return Err("JPEG quality is from 1 to 100".into());
        }
        if self.max_ppi.is_some_and(|p| !(72..=1200).contains(&p)) {
            return Err("pictures keep from 72 to 1200 pixels an inch".into());
        }
        if self.range.is_some_and(|(a, b)| a == 0 || b < a) {
            return Err("a page range runs from a page to a later one, counted from 1".into());
        }
        if self.slug.is_some_and(|s| !(0.0..=144.0).contains(&s)) {
            return Err("the room for crop marks is from 0 to 144 points".into());
        }
        if let Some(BleedChoice::Custom { points }) = self.bleed {
            if !(0.0..=36.0).contains(&points) {
                return Err("a bleed is from 0 to 36 points".into());
            }
        }
        Ok(())
    }
}

/// The choices for each kind of PDF.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase", default)]
pub struct PdfPrefsByTarget {
    #[serde(skip_serializing_if = "PdfPrefs::is_empty")]
    pub sharing: PdfPrefs,
    /// The paperback interior and cover for Kindle Direct Publishing.
    #[serde(skip_serializing_if = "PdfPrefs::is_empty")]
    pub kdp: PdfPrefs,
    #[serde(skip_serializing_if = "PdfPrefs::is_empty")]
    pub print_shop: PdfPrefs,
}

/// What a person chose for their exports.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase", default)]
pub struct ExportPrefs {
    #[serde(skip_serializing_if = "PdfPrefsByTarget::is_empty")]
    pub pdf: PdfPrefsByTarget,
    /// Where the files go, when not the usual `exports` folder. In a book, a folder inside the
    /// book's own is kept relative to it.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub folder: Option<String>,
    /// The files last made, by their names in the Export dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_targets: Option<Vec<String>>,
    /// Keys this version does not know, kept as they were so a save from an older build does not
    /// lose them.
    #[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(skip)]
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

impl PdfPrefsByTarget {
    pub fn is_empty(&self) -> bool {
        self == &PdfPrefsByTarget::default()
    }
}

impl ExportPrefs {
    pub fn is_empty(&self) -> bool {
        self == &ExportPrefs::default()
    }

    /// Why these choices cannot be kept, or `None` when they can.
    pub fn check(&self) -> Result<(), String> {
        self.pdf.sharing.check()?;
        self.pdf.kdp.check()?;
        self.pdf.print_shop.check()?;
        if self.folder.as_ref().is_some_and(|f| f.len() > 4096) {
            return Err("a folder's path is too long".into());
        }
        Ok(())
    }
}

// The only floats are a bleed and a room for marks, which `check` keeps to plain numbers, so equality
// is whole.
impl Eq for BleedChoice {}
impl Eq for PdfPrefs {}
impl Eq for PdfPrefsByTarget {}
impl Eq for ExportPrefs {}
