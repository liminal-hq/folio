// Tests the plugin against the mock runtime, which cannot parent or block a window
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde_json::json;
use tauri::Manager;

use crate::models::DialogSpec;
use crate::session::{self, Dialogs, Options};

fn spec(modal: bool) -> DialogSpec {
    DialogSpec {
        title: "Test".into(),
        width: 400.0,
        height: 300.0,
        min_width: None,
        min_height: None,
        resizable: false,
        modal,
        chromeless: true,
        url: None,
    }
}

fn app() -> tauri::App<tauri::test::MockRuntime> {
    let app = tauri::test::mock_app();
    app.manage(Dialogs::new(Options::default()));
    app
}

#[test]
fn a_dialog_opens_as_a_window_of_its_own_and_reopening_makes_no_second() {
    let app = app();
    let first = session::open(
        app.handle(),
        None,
        "dialog-a",
        &spec(false),
        json!({"n": 1}),
    )
    .expect("opens");
    assert!(!first.existing);
    let again =
        session::open(app.handle(), None, "dialog-a", &spec(false), json!(null)).expect("focuses");
    assert!(again.existing);
    assert_eq!(again.request_id, first.request_id);
    assert!(app.get_webview_window("dialog-a").is_some());
    assert_eq!(app.webview_windows().len(), 1);
}

#[test]
fn the_dialog_can_read_what_it_was_opened_with() {
    let app = app();
    session::open(app.handle(), None, "dialog-a", &spec(true), json!({"n": 1})).expect("opens");
    let state = app.state::<Dialogs>();
    let request = state.registry.lock().unwrap().request("dialog-a").unwrap();
    assert_eq!(request.payload, json!({"n": 1}));
    assert!(request.modal);
}

#[test]
fn ending_a_dialog_frees_it_and_a_second_end_does_nothing() {
    let app = app();
    session::open(app.handle(), None, "dialog-a", &spec(true), json!(null)).expect("opens");
    session::ended(app.handle(), "dialog-a", Some(json!("ok")));
    session::ended(app.handle(), "dialog-a", None);
    assert!(app.state::<Dialogs>().registry.lock().unwrap().is_empty());
}

#[test]
fn posting_from_something_that_is_not_a_dialog_fails() {
    let app = app();
    assert!(session::post(app.handle(), "main", json!(1)).is_err());
}
