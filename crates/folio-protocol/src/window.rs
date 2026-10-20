// Window kinds, matching the webview labels the frontend routes on.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The label of the main window, which holds the open books.
pub const MAIN_LABEL: &str = "main";
/// The label of the About Folio window.
pub const ABOUT_LABEL: &str = "about";
/// The label of the Preferences window.
pub const PREFERENCES_LABEL: &str = "preferences";
/// The label of the Book tools window, the utility window that hosts one workbench at a time.
pub const BOOK_TOOLS_LABEL: &str = "book-tools";
/// The label of the New book wizard window.
pub const WIZARD_NEW_LABEL: &str = "wizard-new";

/// The kind of window a webview label denotes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum WindowKind {
    Main,
    About,
    Preferences,
    BookTools,
    WizardNew,
}

impl WindowKind {
    /// Classifies a webview label, or `None` if it is not a known window.
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            MAIN_LABEL => Some(Self::Main),
            ABOUT_LABEL => Some(Self::About),
            PREFERENCES_LABEL => Some(Self::Preferences),
            BOOK_TOOLS_LABEL => Some(Self::BookTools),
            WIZARD_NEW_LABEL => Some(Self::WizardNew),
            _ => None,
        }
    }

    /// The webview label of the one window of this kind.
    pub fn label(self) -> &'static str {
        match self {
            Self::Main => MAIN_LABEL,
            Self::About => ABOUT_LABEL,
            Self::Preferences => PREFERENCES_LABEL,
            Self::BookTools => BOOK_TOOLS_LABEL,
            Self::WizardNew => WIZARD_NEW_LABEL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_round_trips_through_its_label() {
        for kind in [
            WindowKind::Main,
            WindowKind::About,
            WindowKind::Preferences,
            WindowKind::BookTools,
            WindowKind::WizardNew,
        ] {
            assert_eq!(WindowKind::from_label(kind.label()), Some(kind));
        }
    }

    #[test]
    fn rejects_unknown_labels() {
        assert_eq!(WindowKind::from_label("mystery"), None);
    }
}
