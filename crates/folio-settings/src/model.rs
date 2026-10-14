// The settings document's types and the rules a value must satisfy
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use ts_rs::TS;

/// The narrowest the sidebar can be dragged, in logical pixels (spec 12).
pub const SIDEBAR_WIDTH_MIN: u32 = 240;
/// The widest the sidebar can be dragged, in logical pixels.
pub const SIDEBAR_WIDTH_MAX: u32 = 560;
/// How many recently opened books are remembered.
pub const RECENT_BOOKS_MAX: usize = 20;

/// How the person likes to work (D-01): remembered per user, never per book, and never removes data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum Mode {
    #[default]
    Easy,
    Expert,
}

/// Which colour scheme the app uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum ColourMode {
    /// Follow the system's light or dark setting.
    #[default]
    System,
    Light,
    Dark,
}

/// The unit lengths are shown in. Stored lengths are always points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum Units {
    #[default]
    Inches,
    Millimetres,
}

/// The unit the rulers count in. It is kept apart from `Units`, which the inspector uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum RulerUnit {
    #[default]
    Inches,
    Millimetres,
    Centimetres,
    Points,
    Picas,
    /// CSS pixels, 96 to the inch.
    Pixels,
}

/// Whether the rulers show along the top and left of the spread, and what they count in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct RulerSettings {
    pub on: bool,
    pub units: RulerUnit,
}

/// How much Jax, the optional helper, says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum JaxHelp {
    /// A step-by-step walk through the whole book.
    Guide,
    /// Gentle tips and the page checklist.
    #[default]
    Tips,
    /// Jax is switched off completely.
    Off,
}

/// Which toolbar sits under the title bar (D-07).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum ToolbarLayout {
    #[default]
    Ribbon,
    Classic,
}

/// Where the sidebar lives (spec 12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum SidebarLayout {
    /// A floating card on the right with grouped tabs.
    #[default]
    RightGrouped,
    /// A tool rail on the left beside an inspector on the right.
    RailAndInspector,
    /// A narrow icon rail whose panels fly out over the canvas.
    IconRail,
}

/// The interface choices the app remembers between runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct InterfaceSettings {
    /// Whether the menu bar is shown. Alt or F10 still reveals it while it is hidden.
    pub menu_bar: bool,
    pub toolbar: ToolbarLayout,
    pub sidebar_layout: SidebarLayout,
    /// The sidebar's width in logical pixels, between `SIDEBAR_WIDTH_MIN` and `SIDEBAR_WIDTH_MAX`.
    pub sidebar_width: u32,
    /// Whether the sidebar is collapsed to its icon rail.
    pub sidebar_collapsed: bool,
    /// Whether controls are 44 px instead of 28 px.
    pub larger_controls: bool,
}

impl Default for InterfaceSettings {
    fn default() -> Self {
        Self {
            menu_bar: false,
            toolbar: ToolbarLayout::default(),
            sidebar_layout: SidebarLayout::default(),
            sidebar_width: 292,
            sidebar_collapsed: false,
            larger_controls: false,
        }
    }
}

/// How many curated colours each page mark can be drawn in (the index is `0..MARK_COLOURS`).
pub const MARK_COLOURS: u8 = 4;

/// One page mark: whether it is drawn, and which of its four curated colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct MarkSetting {
    pub on: bool,
    /// An index into the mark's curated colours, `0` to `3`.
    pub colour: u8,
}

impl MarkSetting {
    const fn new(on: bool) -> Self {
        Self { on, colour: 0 }
    }
}

impl Default for MarkSetting {
    fn default() -> Self {
        Self::new(false)
    }
}

/// The marks drawn over a page (spec 06): none of them prints or exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct MarksSettings {
    pub bleed: MarkSetting,
    pub margins: MarkSetting,
    pub columns: MarkSetting,
    pub baseline: MarkSetting,
    pub frame_edges: MarkSetting,
    /// The ruler guides.
    pub guides: MarkSetting,
    /// Whether dragging snaps to the page's edges, margins and other frames.
    pub snap: bool,
}

impl Default for MarksSettings {
    fn default() -> Self {
        Self {
            bleed: MarkSetting::new(true),
            margins: MarkSetting::new(true),
            columns: MarkSetting::new(false),
            baseline: MarkSetting::new(false),
            frame_edges: MarkSetting::new(false),
            guides: MarkSetting::new(true),
            snap: true,
        }
    }
}

impl MarksSettings {
    fn each(&mut self) -> [(&'static str, &mut MarkSetting); 6] {
        [
            ("marks.bleed.colour", &mut self.bleed),
            ("marks.margins.colour", &mut self.margins),
            ("marks.columns.colour", &mut self.columns),
            ("marks.baseline.colour", &mut self.baseline),
            ("marks.frameEdges.colour", &mut self.frame_edges),
            ("marks.guides.colour", &mut self.guides),
        ]
    }
}

/// The application settings: one document every window reads.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Settings {
    pub mode: Mode,
    pub colour_mode: ColourMode,
    pub units: Units,
    pub jax: JaxHelp,
    pub interface: InterfaceSettings,
    pub marks: MarksSettings,
    pub rulers: RulerSettings,
    /// The paths of recently opened books, newest first.
    pub recent_books: Vec<String>,
}

/// A value the settings refuse.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SettingsError {
    #[error("{field} must be between {min} and {max}")]
    OutOfRange {
        /// The setting, as the page names it (`interface.sidebarWidth`).
        field: &'static str,
        min: u32,
        max: u32,
    },
    /// A change that is not shaped like the settings.
    #[error("the change is not valid: {0}")]
    Malformed(String),
}

impl Settings {
    /// Checks every value against its range.
    pub fn validate(&self) -> Result<(), SettingsError> {
        let width = self.interface.sidebar_width;
        if !(SIDEBAR_WIDTH_MIN..=SIDEBAR_WIDTH_MAX).contains(&width) {
            return Err(SettingsError::OutOfRange {
                field: "interface.sidebarWidth",
                min: SIDEBAR_WIDTH_MIN,
                max: SIDEBAR_WIDTH_MAX,
            });
        }
        let mut marks = self.marks;
        for (field, mark) in marks.each() {
            if mark.colour >= MARK_COLOURS {
                return Err(SettingsError::OutOfRange {
                    field,
                    min: 0,
                    max: u32::from(MARK_COLOURS) - 1,
                });
            }
        }
        if self.recent_books.len() > RECENT_BOOKS_MAX {
            return Err(SettingsError::OutOfRange {
                field: "recentBooks",
                min: 0,
                max: RECENT_BOOKS_MAX as u32,
            });
        }
        Ok(())
    }

    /// The settings with every out-of-range value brought to the nearest valid one, for a stored
    /// document edited by hand: a bad value must not stop the app from starting.
    pub fn clamped(mut self) -> Self {
        self.interface.sidebar_width = self
            .interface
            .sidebar_width
            .clamp(SIDEBAR_WIDTH_MIN, SIDEBAR_WIDTH_MAX);
        for (_, mark) in self.marks.each() {
            mark.colour = mark.colour.min(MARK_COLOURS - 1);
        }
        self.recent_books.truncate(RECENT_BOOKS_MAX);
        self
    }

    /// These settings with `patch` laid over them, as a JSON merge patch (RFC 7386): an object
    /// merges key by key, anything else replaces, and `null` removes a key (so a field falls back
    /// to its default). The result is validated; nothing is returned for a patch that does not fit.
    pub fn patched(&self, patch: &Value) -> Result<Settings, SettingsError> {
        let mut document =
            serde_json::to_value(self).map_err(|e| SettingsError::Malformed(e.to_string()))?;
        merge(&mut document, patch);
        let settings: Settings = serde_json::from_value(document)
            .map_err(|e| SettingsError::Malformed(e.to_string()))?;
        settings.validate()?;
        Ok(settings)
    }
}

/// `path` put at the front of `recent`, with any earlier copy removed and the list kept to
/// `RECENT_BOOKS_MAX`: the order the recents list is shown in.
pub fn push_recent(recent: &[String], path: &str) -> Vec<String> {
    let mut list = Vec::with_capacity(recent.len() + 1);
    list.push(path.to_owned());
    list.extend(recent.iter().filter(|p| p.as_str() != path).cloned());
    list.truncate(RECENT_BOOKS_MAX);
    list
}

/// RFC 7386 merge.
fn merge(target: &mut Value, patch: &Value) {
    let Value::Object(changes) = patch else {
        *target = patch.clone();
        return;
    };
    if !target.is_object() {
        *target = Value::Object(Default::default());
    }
    let Some(object) = target.as_object_mut() else {
        return;
    };
    for (key, change) in changes {
        if change.is_null() {
            object.remove(key);
        } else {
            merge(object.entry(key.clone()).or_insert(Value::Null), change);
        }
    }
}

/// The settings with the revision that produced them, so a window can tell a newer state from an
/// older one that arrives late.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct SettingsSnapshot {
    /// Starts at 0 for the settings loaded at start-up and grows by one on every change.
    #[ts(type = "number")]
    pub revision: u64,
    pub settings: Settings,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_defaults_are_the_documented_ones() {
        let s = Settings::default();
        assert_eq!(s.mode, Mode::Easy);
        assert_eq!(s.colour_mode, ColourMode::System);
        assert_eq!(s.units, Units::Inches);
        assert_eq!(s.jax, JaxHelp::Tips);
        assert!(!s.interface.menu_bar);
        assert_eq!(s.interface.toolbar, ToolbarLayout::Ribbon);
        assert_eq!(s.interface.sidebar_width, 292);
        assert!(s.validate().is_ok());
    }

    #[test]
    fn a_document_missing_sections_loads_with_defaults() {
        let s: Settings = serde_json::from_value(json!({ "mode": "expert" })).unwrap();
        assert_eq!(s.mode, Mode::Expert);
        assert_eq!(s.interface, InterfaceSettings::default());
    }

    #[test]
    fn a_patch_changes_only_what_it_names() {
        let before = Settings::default();
        let after = before
            .patched(&json!({ "interface": { "menuBar": true }, "units": "millimetres" }))
            .unwrap();
        assert!(after.interface.menu_bar);
        assert_eq!(after.units, Units::Millimetres);
        assert_eq!(
            after.interface.sidebar_width,
            before.interface.sidebar_width
        );
        assert_eq!(after.mode, before.mode);
    }

    #[test]
    fn null_in_a_patch_restores_the_default() {
        let expert = Settings {
            mode: Mode::Expert,
            ..Settings::default()
        };
        let after = expert.patched(&json!({ "mode": null })).unwrap();
        assert_eq!(after.mode, Mode::Easy);
    }

    #[test]
    fn a_patch_out_of_range_is_refused() {
        let err = Settings::default()
            .patched(&json!({ "interface": { "sidebarWidth": 9000 } }))
            .unwrap_err();
        assert_eq!(
            err,
            SettingsError::OutOfRange {
                field: "interface.sidebarWidth",
                min: SIDEBAR_WIDTH_MIN,
                max: SIDEBAR_WIDTH_MAX
            }
        );
    }

    #[test]
    fn a_patch_of_the_wrong_shape_is_refused() {
        let err = Settings::default()
            .patched(&json!({ "mode": "wizard" }))
            .unwrap_err();
        assert!(matches!(err, SettingsError::Malformed(_)));
    }

    #[test]
    fn a_recent_book_goes_to_the_front_once_and_the_list_stays_short() {
        let list = push_recent(&["/a".into(), "/b".into(), "/c".into()], "/b");
        assert_eq!(list, ["/b", "/a", "/c"]);
        let full: Vec<String> = (0..RECENT_BOOKS_MAX).map(|n| format!("/book{n}")).collect();
        let list = push_recent(&full, "/new");
        assert_eq!(list.len(), RECENT_BOOKS_MAX);
        assert_eq!(list[0], "/new");
        assert!(!list.contains(&format!("/book{}", RECENT_BOOKS_MAX - 1)));
    }

    #[test]
    fn clamping_brings_a_hand_edited_document_into_range() {
        let mut s = Settings::default();
        s.interface.sidebar_width = 10;
        s.recent_books = (0..30).map(|n| format!("/books/{n}.folio")).collect();
        let s = s.clamped();
        assert_eq!(s.interface.sidebar_width, SIDEBAR_WIDTH_MIN);
        assert_eq!(s.recent_books.len(), RECENT_BOOKS_MAX);
        assert!(s.validate().is_ok());
    }

    #[test]
    fn the_marks_start_with_the_bleed_and_margins_on_and_snapping_on() {
        let marks = Settings::default().marks;
        assert!(marks.bleed.on && marks.margins.on && marks.snap);
        assert!(!marks.columns.on && !marks.baseline.on && !marks.frame_edges.on);
        assert!(
            marks.guides.on,
            "guides show once there are rulers to make them with"
        );
        assert_eq!(marks.bleed.colour, 0);
    }

    #[test]
    fn the_rulers_start_off_in_inches_and_are_changed_by_a_patch() {
        let rulers = Settings::default().rulers;
        assert!(!rulers.on);
        assert_eq!(rulers.units, RulerUnit::Inches);
        let patched = Settings::default()
            .patched(&json!({ "rulers": { "on": true, "units": "picas" } }))
            .unwrap();
        assert!(patched.rulers.on);
        assert_eq!(patched.rulers.units, RulerUnit::Picas);
        assert!(Settings::default()
            .patched(&json!({ "rulers": { "units": "cubits" } }))
            .is_err());
    }

    #[test]
    fn a_mark_is_changed_by_a_patch_that_names_only_what_changes() {
        let patched = Settings::default()
            .patched(&json!({ "marks": { "columns": { "on": true, "colour": 2 } } }))
            .unwrap();
        assert!(patched.marks.columns.on);
        assert_eq!(patched.marks.columns.colour, 2);
        assert!(patched.marks.bleed.on, "the others are left alone");
    }

    #[test]
    fn a_colour_past_the_fourth_is_refused_and_clamped_when_read() {
        let refused = Settings::default()
            .patched(&json!({ "marks": { "margins": { "colour": 4 } } }))
            .unwrap_err();
        assert!(matches!(
            refused,
            SettingsError::OutOfRange {
                field: "marks.margins.colour",
                min: 0,
                max: 3
            }
        ));
        let mut stored = Settings::default();
        stored.marks.baseline.colour = 9;
        assert_eq!(stored.clamped().marks.baseline.colour, 3);
    }
}
