// Dialog kinds: the dialogs that open as windows of their own, matching their webview labels.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The start of every dialog window's label (`dialog-export`).
pub const DIALOG_LABEL_PREFIX: &str = "dialog-";

/// A dialog that opens as a window of its own (D-77). It is separate from `WindowKind`: a window
/// kind is one window for the whole app, while a dialog belongs to the window that opened it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum DialogKind {
    Export,
    BookDetails,
    Barcode,
    BookSize,
    Margins,
}

impl DialogKind {
    /// Every kind, for tests and for building capability lists.
    pub const ALL: [DialogKind; 5] = [
        DialogKind::Export,
        DialogKind::BookDetails,
        DialogKind::Barcode,
        DialogKind::BookSize,
        DialogKind::Margins,
    ];

    /// The webview label of the window for this kind.
    pub fn label(self) -> &'static str {
        match self {
            Self::Export => "dialog-export",
            Self::BookDetails => "dialog-book-details",
            Self::Barcode => "dialog-barcode",
            Self::BookSize => "dialog-book-size",
            Self::Margins => "dialog-margins",
        }
    }

    /// Classifies a webview label, or `None` if it is not a dialog window.
    pub fn from_label(label: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.label() == label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_round_trips_through_its_label_and_has_the_prefix() {
        for kind in DialogKind::ALL {
            assert!(kind.label().starts_with(DIALOG_LABEL_PREFIX));
            assert_eq!(DialogKind::from_label(kind.label()), Some(kind));
        }
    }

    #[test]
    fn a_window_label_is_not_a_dialog() {
        assert_eq!(DialogKind::from_label("main"), None);
        assert_eq!(DialogKind::from_label("dialog-mystery"), None);
    }
}
