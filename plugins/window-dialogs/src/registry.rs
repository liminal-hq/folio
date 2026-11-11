// The pure record of which dialogs are open, who opened them and what they were asked
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

use crate::models::DialogRequest;

/// An open dialog.
#[derive(Debug)]
struct Entry {
    request_id: u64,
    opener: Option<String>,
    modal: bool,
    payload: Value,
    opened: Instant,
}

/// What the registry says about a request to open a dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opening {
    /// A new dialog: make its window.
    New { request_id: u64 },
    /// The dialog is already open: bring its window forward.
    Existing { request_id: u64 },
    /// The opener already has a modal dialog (the label) open, so another may not open.
    Refused { modal: String },
}

/// A dialog taken out of the registry because it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Closing {
    pub request_id: u64,
    pub opener: Option<String>,
    pub modal: bool,
}

#[derive(Debug, Default)]
pub struct Registry {
    next: u64,
    open: HashMap<String, Entry>,
}

impl Registry {
    /// Records a dialog about to open, or says why it may not.
    pub fn begin(
        &mut self,
        label: &str,
        opener: Option<&str>,
        modal: bool,
        payload: Value,
    ) -> Opening {
        if let Some(entry) = self.open.get(label) {
            return Opening::Existing {
                request_id: entry.request_id,
            };
        }
        if modal {
            if let Some(opener) = opener {
                if let Some(other) = self.modal_of(opener) {
                    return Opening::Refused {
                        modal: other.to_owned(),
                    };
                }
            }
        }
        self.next += 1;
        let request_id = self.next;
        self.open.insert(
            label.to_owned(),
            Entry {
                request_id,
                opener: opener.map(str::to_owned),
                modal,
                payload,
                opened: Instant::now(),
            },
        );
        Opening::New { request_id }
    }

    /// The label of the modal dialog blocking `opener`, if there is one.
    pub fn modal_of(&self, opener: &str) -> Option<&str> {
        self.open
            .iter()
            .find(|(_, e)| e.modal && e.opener.as_deref() == Some(opener))
            .map(|(label, _)| label.as_str())
    }

    /// What the dialog `label` was asked, for the dialog itself.
    pub fn request(&self, label: &str) -> Option<DialogRequest> {
        self.open.get(label).map(|e| DialogRequest {
            request_id: e.request_id,
            label: label.to_owned(),
            payload: e.payload.clone(),
            modal: e.modal,
        })
    }

    /// The opener of the dialog `label`.
    pub fn opener_of(&self, label: &str) -> Option<(u64, Option<String>)> {
        self.open
            .get(label)
            .map(|e| (e.request_id, e.opener.clone()))
    }

    /// Milliseconds since the dialog was asked for.
    pub fn age_ms(&self, label: &str) -> Option<u128> {
        self.open.get(label).map(|e| e.opened.elapsed().as_millis())
    }

    /// Takes the dialog out of the registry. Whoever gets `Some` ends the dialog and tells the
    /// opener, so a dialog that finished and then closed is reported once.
    pub fn take(&mut self, label: &str) -> Option<Closing> {
        self.open.remove(label).map(|e| Closing {
            request_id: e.request_id,
            opener: e.opener,
            modal: e.modal,
        })
    }

    /// Whether any dialog is open.
    pub fn is_empty(&self) -> bool {
        self.open.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn each_dialog_gets_its_own_request_id() {
        let mut registry = Registry::default();
        let a = registry.begin("dialog-a", Some("main"), false, json!(1));
        let b = registry.begin("dialog-b", Some("main"), false, json!(2));
        assert_eq!(a, Opening::New { request_id: 1 });
        assert_eq!(b, Opening::New { request_id: 2 });
    }

    #[test]
    fn opening_an_open_dialog_again_brings_it_forward() {
        let mut registry = Registry::default();
        registry.begin("dialog-a", Some("main"), true, json!(1));
        assert_eq!(
            registry.begin("dialog-a", Some("main"), true, json!(2)),
            Opening::Existing { request_id: 1 }
        );
        // The first payload stays.
        assert_eq!(registry.request("dialog-a").unwrap().payload, json!(1));
    }

    #[test]
    fn a_window_has_one_modal_dialog_at_a_time() {
        let mut registry = Registry::default();
        registry.begin("dialog-a", Some("main"), true, json!(null));
        assert_eq!(
            registry.begin("dialog-b", Some("main"), true, json!(null)),
            Opening::Refused {
                modal: "dialog-a".into()
            }
        );
        // Another window, or a dialog that does not block, is fine.
        assert!(matches!(
            registry.begin("dialog-b", Some("other"), true, json!(null)),
            Opening::New { .. }
        ));
        assert!(matches!(
            registry.begin("dialog-c", Some("main"), false, json!(null)),
            Opening::New { .. }
        ));
    }

    #[test]
    fn a_dialog_ends_once() {
        let mut registry = Registry::default();
        registry.begin("dialog-a", Some("main"), true, json!(null));
        assert_eq!(
            registry.take("dialog-a"),
            Some(Closing {
                request_id: 1,
                opener: Some("main".into()),
                modal: true
            })
        );
        assert_eq!(registry.take("dialog-a"), None);
        assert!(registry.is_empty());
        // The slot is free again.
        assert!(matches!(
            registry.begin("dialog-b", Some("main"), true, json!(null)),
            Opening::New { request_id: 2 }
        ));
    }

    #[test]
    fn a_dialog_can_be_asked_what_it_was_for() {
        let mut registry = Registry::default();
        registry.begin("dialog-a", None, true, json!({"book": "x"}));
        let request = registry.request("dialog-a").unwrap();
        assert_eq!(request.request_id, 1);
        assert_eq!(request.payload, json!({"book": "x"}));
        assert_eq!(registry.request("dialog-z"), None);
        assert_eq!(registry.opener_of("dialog-a"), Some((1, None)));
    }
}
