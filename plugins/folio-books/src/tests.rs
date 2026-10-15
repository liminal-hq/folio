// Plugin-level tests: the books and their commands run against a mock Tauri app with real windows
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::{Arc, Mutex};

use folio_model::{Intent, Pt, Rect, SpreadId};
use folio_presets::NewBookSpec;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::{Listener, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::{
    commands, init, BookEvent, Books, BooksChanged, Error, PatchEvent, CHANGED_EVENT, PATCH_EVENT,
};

type App = tauri::App<MockRuntime>;

fn app() -> App {
    let app = mock_builder()
        .plugin(init())
        .build(mock_context(noop_assets()))
        .expect("the mock app builds");
    for label in ["main", "preferences"] {
        WebviewWindowBuilder::new(&app, label, WebviewUrl::App("index.html".into()))
            .build()
            .expect("the mock window opens");
    }
    app
}

fn books(app: &App) -> tauri::State<'_, Books<MockRuntime>> {
    app.state::<Books<MockRuntime>>()
}

fn hear<T: serde::de::DeserializeOwned + Send + 'static>(
    app: &App,
    label: &str,
    event: &str,
) -> Arc<Mutex<Vec<T>>> {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let sink = heard.clone();
    app.get_webview_window(label)
        .unwrap()
        .listen(event, move |e| {
            sink.lock()
                .unwrap()
                .push(serde_json::from_str(e.payload()).expect("a payload"));
        });
    heard
}

/// A move of the cover's colour block, which every new book has.
fn nudge(app: &App, id: &str, x: f64) -> Intent {
    let snapshot = books(app).snapshot(id).unwrap();
    let cover = &snapshot.book.spreads[0];
    Intent::MoveFrame {
        spread: SpreadId(cover.id.0.clone()),
        frame: cover.frames[0].base().id.clone(),
        rect: Rect {
            x: Pt(x),
            y: Pt(0.0),
            w: Pt(100.0),
            h: Pt(100.0),
        },
    }
}

#[test]
fn the_six_kinds_are_offered() {
    let app = app();
    assert_eq!(books(&app).presets().len(), 6);
}

#[test]
fn the_palette_is_offered() {
    let app = app();
    let window = app.get_webview_window("main").expect("the main window");
    let paints = tauri::async_runtime::block_on(commands::paint_presets(window)).unwrap();
    assert!(!paints.swatches.is_empty() && !paints.gradients.is_empty());
}

#[test]
fn a_new_book_is_open_but_unsaved_and_nothing_is_written() {
    let app = app();
    let snapshot = books(&app).new_book(&NewBookSpec::default()).unwrap();
    assert_eq!(snapshot.revision, 0);
    assert_eq!(snapshot.summary.path, None);
    assert!(
        snapshot.summary.dirty,
        "a book never saved has something to save"
    );
    assert_eq!(snapshot.book.title, "The Otter Who Couldn't Sleep");
    assert_eq!(books(&app).list().len(), 1);
    assert!(!snapshot.can_undo && !snapshot.can_redo);
}

#[test]
fn every_window_hears_when_books_open_and_close() {
    let app = app();
    let main = hear::<BooksChanged>(&app, "main", CHANGED_EVENT);
    let other = hear::<BooksChanged>(&app, "preferences", CHANGED_EVENT);
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    books(&app).close(&id).unwrap();
    for heard in [main, other] {
        let heard = heard.lock().unwrap();
        assert_eq!(heard.len(), 2);
        assert_eq!(heard[0].books.len(), 1);
        assert!(heard[1].books.is_empty());
    }
}

#[test]
fn an_intent_answers_with_a_patch_and_every_window_hears_it() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let main = hear::<PatchEvent>(&app, "main", PATCH_EVENT);
    let other = hear::<PatchEvent>(&app, "preferences", PATCH_EVENT);

    let patch = books(&app).apply(&id, nudge(&app, &id, 40.0)).unwrap();
    assert_eq!(patch.revision, 1);
    for heard in [main, other] {
        let heard = heard.lock().unwrap();
        assert_eq!(heard.len(), 1);
        assert_eq!(heard[0].id, id);
        assert_eq!(heard[0].patch, patch);
        assert!(heard[0].can_undo && !heard[0].can_redo);
    }
}

#[test]
fn undo_and_redo_send_patches_too_and_say_when_there_is_nothing_to_do() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    assert!(books(&app).undo(&id).unwrap().is_none());
    assert!(books(&app).redo(&id).unwrap().is_none());

    let heard = hear::<PatchEvent>(&app, "main", PATCH_EVENT);
    books(&app).apply(&id, nudge(&app, &id, 40.0)).unwrap();
    let undone = books(&app).undo(&id).unwrap().expect("something to undo");
    let redone = books(&app).redo(&id).unwrap().expect("something to redo");
    assert_eq!((undone.revision, redone.revision), (2, 3));
    assert_eq!(heard.lock().unwrap().len(), 3);
}

#[test]
fn a_refused_intent_is_a_typed_error_and_nothing_is_sent() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let heard = hear::<PatchEvent>(&app, "main", PATCH_EVENT);
    let refused = books(&app)
        .apply(
            &id,
            Intent::DeleteSpread {
                spread: SpreadId(
                    books(&app).snapshot(&id).unwrap().book.spreads[0]
                        .id
                        .0
                        .clone(),
                ),
            },
        )
        .unwrap_err();
    assert_eq!(serde_json::to_value(&refused).unwrap()["kind"], "session");
    assert!(heard.lock().unwrap().is_empty());
    assert_eq!(books(&app).snapshot(&id).unwrap().revision, 0);
}

#[test]
fn an_unknown_book_is_a_typed_error() {
    let app = app();
    let error = books(&app).snapshot("bs_nope").unwrap_err();
    assert!(matches!(error, Error::NoSuchBook(_)));
    assert_eq!(serde_json::to_value(&error).unwrap()["kind"], "noSuchBook");
}

#[test]
fn saving_needs_a_place_the_first_time_then_remembers_it() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    assert!(matches!(books(&app).save(&id, None), Err(Error::NeedsPath)));

    let file = dir.path().join("Otter").join("Otter.folio");
    let saved = books(&app).save(&id, Some(&file)).unwrap();
    assert!(file.exists(), "the folder is made too");
    assert!(!saved.dirty);
    assert_eq!(
        std::fs::canonicalize(saved.path.unwrap()).unwrap(),
        std::fs::canonicalize(&file).unwrap()
    );

    // A change makes it dirty again, and a save with no path goes to the same file.
    books(&app).apply(&id, nudge(&app, &id, 10.0)).unwrap();
    assert!(books(&app).list()[0].dirty);
    assert!(!books(&app).save(&id, None).unwrap().dirty);
}

#[test]
fn a_saved_book_opens_again_with_everything_it_had() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("kept.folio");
    let spec = NewBookSpec {
        title: "Kept".into(),
        ..NewBookSpec::default()
    };
    let id = books(&app).new_book(&spec).unwrap().summary.id;
    books(&app).apply(&id, nudge(&app, &id, 77.0)).unwrap();
    books(&app).save(&id, Some(&file)).unwrap();
    books(&app).close(&id).unwrap();

    let reopened = books(&app).open(&file).unwrap();
    assert_eq!(reopened.book.title, "Kept");
    assert_eq!(reopened.revision, 0);
    assert!(!reopened.summary.dirty);
    assert_eq!(
        reopened.settings["paper"], "white",
        "the book's own settings travel with it"
    );
    assert!(reopened.styles["styles"].get("display").is_some());
    let moved = reopened.book.spreads[0].frames[0].base().rect.x;
    assert_eq!(moved, Pt(77.0));
}

#[test]
fn opening_a_book_that_is_already_open_gives_the_same_session() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("once.folio");
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    books(&app).save(&id, Some(&file)).unwrap();
    let again = books(&app).open(&file).unwrap();
    assert_eq!(again.summary.id, id);
    assert_eq!(books(&app).list().len(), 1);
}

#[test]
fn a_file_that_is_not_a_book_is_refused_with_a_reason() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let text = dir.path().join("notes.folio");
    std::fs::write(&text, "just words").unwrap();
    let error = books(&app).open(&text).unwrap_err();
    assert_eq!(serde_json::to_value(&error).unwrap()["kind"], "package");
    assert!(books(&app).list().is_empty());
}

#[test]
fn the_app_hears_when_a_book_opens_saves_or_closes() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("heard.folio");
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = seen.clone();
    books(&app).on_book(move |event| {
        sink.lock().unwrap().push(match event {
            BookEvent::Opened(s) => format!("opened {}", s.title),
            BookEvent::Saved(s) => format!("saved {}", s.title),
            BookEvent::Closed(_) => "closed".into(),
        });
    });
    let spec = NewBookSpec {
        title: "Heard".into(),
        ..NewBookSpec::default()
    };
    let id = books(&app).new_book(&spec).unwrap().summary.id;
    books(&app).save(&id, Some(&file)).unwrap();
    books(&app).close(&id).unwrap();
    books(&app).open(&file).unwrap();
    // A new book is not "opened" from a file; saving, closing and reopening are.
    assert_eq!(
        *seen.lock().unwrap(),
        ["saved Heard", "closed", "opened Heard"]
    );
}

#[test]
fn the_default_place_for_a_book_is_a_folder_of_its_own() {
    let app = app();
    let path = books(&app).default_save_path("Pip & Moon: part 1");
    let text = path.to_string_lossy();
    assert!(
        text.ends_with("Books/Pip & Moon part 1/Pip & Moon part 1.folio")
            || text.ends_with("Books\\Pip & Moon part 1\\Pip & Moon part 1.folio"),
        "{text}"
    );
}

#[test]
fn the_status_command_reports_the_plugin_available() {
    let status = tauri::async_runtime::block_on(commands::get_status());
    assert!(status.available);
    assert_eq!(status.features, vec!["books", "events"]);
}

#[test]
fn a_spread_is_laid_out_at_the_revision_the_book_is_at() {
    let app = app();
    let snapshot = books(&app).new_book(&NewBookSpec::default()).unwrap();
    let spread = snapshot.book.spreads[1].id.0.clone();
    let before = books(&app)
        .layout_spread(&snapshot.summary.id, &spread)
        .unwrap();
    assert_eq!(before.revision, snapshot.revision);
    assert_eq!(before.layout.spread.0, spread);
    assert!(before.layout.frames.iter().any(|f| !f.lines.is_empty()));

    books(&app)
        .apply(&snapshot.summary.id, nudge(&app, &snapshot.summary.id, 5.0))
        .unwrap();
    let after = books(&app)
        .layout_spread(&snapshot.summary.id, &spread)
        .unwrap();
    assert_eq!(after.revision, before.revision + 1);
}

#[test]
fn laying_out_an_unknown_book_or_spread_is_a_typed_error() {
    let app = app();
    let snapshot = books(&app).new_book(&NewBookSpec::default()).unwrap();
    let missing = books(&app).layout_spread("nope", "x").unwrap_err();
    assert_eq!(
        serde_json::to_value(&missing).unwrap()["kind"],
        "noSuchBook"
    );
    let spread = books(&app)
        .layout_spread(&snapshot.summary.id, "no-such-spread")
        .unwrap_err();
    assert_eq!(
        serde_json::to_value(&spread).unwrap()["kind"],
        "noSuchSpread"
    );
}

#[test]
fn a_picture_file_goes_into_the_book_and_comes_back_by_its_path() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Otter.PNG");
    let png = [b"\x89PNG\r\n\x1a\n".as_slice(), b"and some bytes"].concat();
    std::fs::write(&file, &png).unwrap();
    let asset = books(&app).import_image(&id, &file).unwrap();
    assert!(
        asset.starts_with("assets/images/") && asset.ends_with(".png"),
        "{asset}"
    );
    assert_eq!(books(&app).asset_data(&id, &asset).unwrap(), png);
    // The same bytes are stored once.
    assert_eq!(books(&app).import_image(&id, &file).unwrap(), asset);
}

#[test]
fn a_file_that_is_not_a_picture_is_refused_with_a_typed_error() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("notes.txt");
    std::fs::write(&file, b"hello").unwrap();
    let refused = books(&app).import_image(&id, &file).unwrap_err();
    assert_eq!(serde_json::to_value(&refused).unwrap()["kind"], "image");
    let missing = books(&app)
        .import_image(&id, &dir.path().join("gone.png"))
        .unwrap_err();
    assert_eq!(serde_json::to_value(&missing).unwrap()["kind"], "image");
    assert!(books(&app)
        .asset_data(&id, "assets/images/nope.png")
        .is_err());
    assert!(matches!(
        books(&app).import_image("bk_none", &dir.path().join("a.png")),
        Err(Error::BadImage(_) | Error::NoSuchBook(_))
    ));
}

#[test]
fn a_file_that_is_not_what_its_name_says_and_a_link_are_refused() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let dir = tempfile::tempdir().unwrap();
    let renamed = dir.path().join("really-text.png");
    std::fs::write(&renamed, b"just words").unwrap();
    let refused = books(&app).import_image(&id, &renamed).unwrap_err();
    assert_eq!(serde_json::to_value(&refused).unwrap()["kind"], "image");
    #[cfg(unix)]
    {
        let real = dir.path().join("real.png");
        std::fs::write(&real, b"\x89PNG\r\n\x1a\nbytes").unwrap();
        let link = dir.path().join("link.png");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        assert!(books(&app).import_image(&id, &link).is_err());
        assert!(books(&app).import_image(&id, &real).is_ok());
    }
}
