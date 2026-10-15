// The kind bundles as data, and the parts of them the UI shows
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeMap;
use std::sync::OnceLock;

use folio_model::{BookKind, LayoutMode, Pt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

const KINDS_JSON: &str = include_str!("../data/kinds.json");
const STYLES_JSON: &str = include_str!("../data/styles.json");

/// The order the kinds are offered in.
pub(crate) const KIND_ORDER: [BookKind; 6] = [
    BookKind::Picture,
    BookKind::Novel,
    BookKind::Textbook,
    BookKind::Illustrated,
    BookKind::Poetry,
    BookKind::Memoir,
];

fn key(kind: BookKind) -> &'static str {
    match kind {
        BookKind::Picture => "picture",
        BookKind::Novel => "novel",
        BookKind::Textbook => "textbook",
        BookKind::Illustrated => "illustrated",
        BookKind::Poetry => "poetry",
        BookKind::Memoir => "memoir",
    }
}

/// One frame of a sample spread, in inches in spread space.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(crate) enum SampleFrame {
    Text {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        style: String,
        text: String,
    },
    Image {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        #[allow(dead_code)]
        tint: String,
        label: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SampleSpread {
    pub name: Option<String>,
    #[serde(default)]
    pub single: bool,
    #[serde(default)]
    pub opener: bool,
    pub frames: Vec<SampleFrame>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KindData {
    pub label: String,
    pub sub: String,
    pub sample_title: String,
    pub trims: Vec<String>,
    pub margin_in: f64,
    pub bleed_in: f64,
    pub columns: u8,
    pub flowing: bool,
    pub default_pages: u32,
    #[allow(dead_code)]
    pub folios: bool,
    /// The running head the kind starts with; masters use it once they exist.
    #[allow(dead_code)]
    pub running_head: Option<String>,
    pub styles: Vec<String>,
    pub spreads: Vec<SampleSpread>,
}

fn kinds() -> &'static BTreeMap<String, KindData> {
    static KINDS: OnceLock<BTreeMap<String, KindData>> = OnceLock::new();
    KINDS.get_or_init(|| serde_json::from_str(KINDS_JSON).expect("data/kinds.json is valid"))
}

pub(crate) fn kind_data(kind: BookKind) -> &'static KindData {
    kinds().get(key(kind)).expect("every kind has data")
}

/// Every style Folio ships, as `{ "styles": { id: {...} } }`, to fall back on when a book lacks one.
pub fn preset_styles() -> Value {
    serde_json::json!({ "styles": style_table() })
}

pub(crate) fn style_table() -> &'static BTreeMap<String, Value> {
    static STYLES: OnceLock<BTreeMap<String, Value>> = OnceLock::new();
    STYLES.get_or_init(|| serde_json::from_str(STYLES_JSON).expect("data/styles.json is valid"))
}

/// A trim written `8.5 × 8.5 in`, as inches wide and high.
pub(crate) fn parse_trim(text: &str) -> Option<(f64, f64)> {
    let text = text.trim().trim_end_matches("in").trim();
    let (w, h) = text.split_once(['×', 'x'])?;
    Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
}

/// A size a book of some kind can be, as the UI offers it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct TrimChoice {
    /// How it is written for people: `8.5 × 8.5 in`.
    pub label: String,
    pub width: Pt,
    pub height: Pt,
}

/// A kind of book, as the welcome screen and the wizard show it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct KindInfo {
    pub kind: BookKind,
    pub label: String,
    pub sub: String,
    pub layout: LayoutMode,
    pub trims: Vec<TrimChoice>,
    /// How long a book of this kind usually is, for the spine and the status bar.
    pub default_pages: u32,
    pub sample_title: String,
}

/// The six kinds, in the order they are offered.
pub fn kind_infos() -> Vec<KindInfo> {
    KIND_ORDER
        .iter()
        .map(|&kind| {
            let data = kind_data(kind);
            KindInfo {
                kind,
                label: data.label.clone(),
                sub: data.sub.clone(),
                layout: if data.flowing {
                    LayoutMode::Flowing
                } else {
                    LayoutMode::Fixed
                },
                trims: data
                    .trims
                    .iter()
                    .filter_map(|label| {
                        parse_trim(label).map(|(w, h)| TrimChoice {
                            label: label.clone(),
                            width: Pt::from_in(w),
                            height: Pt::from_in(h),
                        })
                    })
                    .collect(),
                default_pages: data.default_pages,
                sample_title: data.sample_title.clone(),
            }
        })
        .collect()
}
