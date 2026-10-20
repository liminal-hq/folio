// Plugin-level tests: the books and their commands run against a mock Tauri app with real windows
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::wire::{ExportColour, ExportTarget};
use std::sync::{Arc, Mutex};

use folio_model::{Intent, Pt, Rect, SpreadId};
use folio_package::Package;
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
        on: folio_model::Surface::Spread(SpreadId(cover.id.0.clone())),
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
fn rewinding_to_a_mark_undoes_the_steps_since_it_and_announces_each() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    books(&app).apply(&id, nudge(&app, &id, 10.0)).unwrap();
    let mark = books(&app).history_mark(&id).unwrap();
    assert_eq!(mark, 1);

    let heard = hear::<PatchEvent>(&app, "main", PATCH_EVENT);
    // Words ignored are a step each, where moving a frame again would join the step before it.
    for word in ["otter", "reed"] {
        let word = word.to_string();
        books(&app).apply(&id, Intent::IgnoreWord { word }).unwrap();
    }
    assert_eq!(books(&app).history_mark(&id).unwrap(), 3);
    assert_eq!(heard.lock().unwrap().len(), 2);

    books(&app).rewind(&id, mark).unwrap();
    assert_eq!(books(&app).history_mark(&id).unwrap(), mark);
    // Two steps went, and each was announced on its own so a window's copy stays in step.
    assert_eq!(heard.lock().unwrap().len(), 4);
    // Rewinding to where the book already is does nothing.
    books(&app).rewind(&id, mark).unwrap();
    assert_eq!(heard.lock().unwrap().len(), 4);
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
        .layout_spread(&snapshot.summary.id, &spread, None)
        .unwrap();
    assert_eq!(before.revision, snapshot.revision);
    assert_eq!(before.layout.spread.0, spread);
    assert!(before.layout.frames.iter().any(|f| !f.lines.is_empty()));

    books(&app)
        .apply(&snapshot.summary.id, nudge(&app, &snapshot.summary.id, 5.0))
        .unwrap();
    let after = books(&app)
        .layout_spread(&snapshot.summary.id, &spread, None)
        .unwrap();
    assert_eq!(after.revision, before.revision + 1);
}

#[test]
fn a_master_page_is_laid_out_with_its_tokens_as_written() {
    let app = app();
    let snapshot = books(&app).new_book(&NewBookSpec::default()).unwrap();
    let id = snapshot.summary.id.clone();
    books(&app)
        .apply(
            &id,
            Intent::SetPageNumbers {
                position: folio_model::PageNumbers::Centre,
            },
        )
        .unwrap();
    let laid = books(&app).layout_master(&id, "mst_a").unwrap();
    assert_eq!(laid.layout.spread.0, "mst_a");
    assert_eq!(laid.layout.frames.len(), 1);
    let text: String = laid.layout.frames[0]
        .lines
        .iter()
        .flat_map(|l| l.runs.iter().map(|r| r.text.clone()))
        .collect();
    assert!(
        text.contains("{page}"),
        "the token stays as written: {text}"
    );
    let missing = books(&app).layout_master(&id, "mst_nope").unwrap_err();
    assert_eq!(
        serde_json::to_value(&missing).unwrap()["kind"],
        "noSuchMaster"
    );
}

#[test]
fn laying_out_an_unknown_book_or_spread_is_a_typed_error() {
    let app = app();
    let snapshot = books(&app).new_book(&NewBookSpec::default()).unwrap();
    let missing = books(&app).layout_spread("nope", "x", None).unwrap_err();
    assert_eq!(
        serde_json::to_value(&missing).unwrap()["kind"],
        "noSuchBook"
    );
    let spread = books(&app)
        .layout_spread(&snapshot.summary.id, "no-such-spread", None)
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

/// A PNG header that says it is `w` × `h` pixels; enough for the book to read its size.
fn png_of(w: u32, h: u32) -> Vec<u8> {
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    png.extend_from_slice(&w.to_be_bytes());
    png.extend_from_slice(&h.to_be_bytes());
    png
}

#[test]
fn layout_reads_the_size_of_the_pictures_a_page_uses_and_says_when_one_is_missing() {
    use folio_layout::wire::PreflightKind;
    use folio_model::{Background, BackgroundTarget, BgFit};
    let app = app();
    let snapshot = books(&app).new_book(&NewBookSpec::default()).unwrap();
    let id = snapshot.summary.id.clone();
    let cover = snapshot.book.spreads[0].clone();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("big.png");
    std::fs::write(&file, png_of(4000, 4000)).unwrap();
    let asset = books(&app).import_image(&id, &file).unwrap();
    let set = |asset: &str| Intent::SetBackground {
        target: BackgroundTarget::Page {
            spread: cover.id.clone(),
            page: cover.pages[0].id.clone(),
        },
        bg: Some(Background::Picture {
            asset: asset.into(),
            fit: BgFit::Fill,
            focus: (0.5, 0.5),
        }),
    };
    let problems = |app: &App| {
        books(app)
            .layout_spread(&id, &cover.id.0, None)
            .unwrap()
            .layout
            .preflight
            .into_iter()
            .map(|p| p.kind)
            .filter(|k| {
                matches!(
                    k,
                    PreflightKind::PictureMissing
                        | PreflightKind::LowResolution
                        | PreflightKind::VeryLowResolution
                )
            })
            .collect::<Vec<_>>()
    };
    books(&app).apply(&id, set(&asset)).unwrap();
    assert!(
        problems(&app).is_empty(),
        "4000 px across a small page is sharp"
    );
    // A picture the book does not hold, by a path it could.
    books(&app)
        .apply(&id, set("assets/images/0000.png"))
        .unwrap();
    assert_eq!(problems(&app), [PreflightKind::PictureMissing]);
    // A small picture is read too, and not remembered as the big one.
    let small = dir.path().join("small.png");
    std::fs::write(&small, png_of(300, 300)).unwrap();
    let little = books(&app).import_image(&id, &small).unwrap();
    books(&app).apply(&id, set(&little)).unwrap();
    assert_eq!(problems(&app), [PreflightKind::VeryLowResolution]);
}

#[test]
fn spelling_checks_a_text_against_the_books_dictionary_and_its_ignore_list() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let text = "A small ottr named Zorblat.";
    let found = books(&app).spell_check(&id, text, &[]).unwrap();
    assert!(found.supported);
    let words: Vec<_> = found.misspellings.iter().map(|m| m.word.as_str()).collect();
    assert_eq!(words, ["ottr", "Zorblat"]);
    assert_eq!(found.misspellings[0].start, 8);
    assert_eq!(found.misspellings[0].end, 12);
    // A word the book ignores is no longer flagged, and undoing brings it back.
    books(&app)
        .apply(
            &id,
            Intent::IgnoreWord {
                word: "Zorblat".into(),
            },
        )
        .unwrap();
    let after = books(&app).spell_check(&id, text, &[]).unwrap();
    assert_eq!(after.misspellings.len(), 1);
    books(&app).undo(&id).unwrap();
    assert_eq!(
        books(&app)
            .spell_check(&id, text, &[])
            .unwrap()
            .misspellings
            .len(),
        2
    );
    // A word for every book is left alone too, and the book's own list is not touched by it.
    let mine = books(&app)
        .spell_check(&id, text, &["Zorblat".to_owned(), "ottr".to_owned()])
        .unwrap();
    assert!(mine.misspellings.is_empty());
    let suggestions = books(&app).spell_suggest(&id, "ottr").unwrap();
    assert!(suggestions.iter().any(|s| s == "otter"), "{suggestions:?}");
}

#[test]
fn english_is_checked_with_the_bundled_dictionary_until_a_better_one_is_downloaded() {
    let app = app();
    let books = books(&app);
    for language in ["en", "en-CA", "en_GB", "EN-us"] {
        let (id, _) = books.spelling.for_language(language).expect(language);
        assert_eq!(id, "en-CA", "{language}");
    }
    for language in ["fr-CA", "de", "", "eng"] {
        assert!(
            books.spelling.for_language(language).is_none(),
            "{language}"
        );
    }
    // A dictionary is on offer for a language that has one to download, and not for the bundled one.
    assert_eq!(books.spelling.offer("fr-CA").unwrap().id, "fr");
    assert_eq!(books.spelling.offer("en-GB").unwrap().id, "en-GB");
    assert!(books.spelling.offer("en-CA").is_none());
    assert!(books.spelling.offer("ja").is_none());
}

#[test]
fn the_dictionary_list_has_the_bundled_one_first_and_the_rest_not_yet_in_place() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    books(&app)
        .spelling
        .keep_in(dir.path().join("dictionaries"));
    let list = books(&app).spelling.status();
    assert!(list[0].bundled && list[0].installed && list[0].id == "en-CA");
    assert!(list.len() > 3);
    assert!(list[1..]
        .iter()
        .all(|d| !d.bundled && !d.installed && d.bytes > 0));
    assert!(list
        .iter()
        .any(|d| d.id == "fr" && d.languages.contains(&"fr-CA".to_owned())));
    // Nothing is downloaded for an id the catalogue does not hold, and nothing is written.
    assert!(matches!(
        books(&app).spelling.download("../../etc", &|_, _| {}),
        Err(Error::Dictionary(_))
    ));
    assert!(matches!(
        books(&app).spelling.remove("nope"),
        Err(Error::Dictionary(_))
    ));
    assert!(!dir.path().join("dictionaries").join("nope").exists());
}

#[test]
fn spelling_for_an_unknown_book_is_a_typed_error() {
    let app = app();
    assert!(matches!(
        books(&app).spell_check("nope", "x", &[]),
        Err(Error::NoSuchBook(_))
    ));
    assert!(matches!(
        books(&app).spell_suggest("nope", "x"),
        Err(Error::NoSuchBook(_))
    ));
}

/// Run by hand (`cargo test -p tauri-plugin-folio-books -- --ignored`): the real download, checked
/// against the pinned hashes, and used to check a word.
#[test]
#[ignore = "needs the network"]
fn a_dictionary_downloads_checks_and_checks_words() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    books(&app)
        .spelling
        .keep_in(dir.path().join("dictionaries"));
    let told = std::cell::RefCell::new(Vec::new());
    books(&app)
        .spelling
        .download("fr", &|received, total| {
            told.borrow_mut().push((received, total))
        })
        .unwrap();
    // Progress comes as the bytes arrive, never goes back, and ends with all of them.
    let told = told.into_inner();
    assert!(told.len() > 2, "{told:?}");
    assert!(told.windows(2).all(|w| w[0].0 <= w[1].0));
    let (last, total) = *told.last().unwrap();
    assert_eq!(last, total);
    assert!(told.iter().all(|&(_, t)| t == total));
    let list = books(&app).spelling.status();
    assert!(list.iter().any(|d| d.id == "fr" && d.installed));
    let (id, french) = books(&app).spelling.for_language("fr-CA").unwrap();
    assert_eq!(id, "fr");
    assert!(folio_spell::Checker::check(&*french, "bonjour"));
    assert!(!folio_spell::Checker::check(&*french, "bonjuor"));
    assert!(dir.path().join("dictionaries/fr/LICENSE").is_file());
    books(&app).spelling.remove("fr").unwrap();
    assert!(books(&app).spelling.for_language("fr-CA").is_none());
}

#[test]
fn layout_marks_misspelt_words_only_when_asked_and_leaves_ignored_ones_alone() {
    let app = app();
    let snapshot = books(&app).new_book(&NewBookSpec::default()).unwrap();
    let id = snapshot.summary.id.clone();
    let spread = snapshot.book.spreads[1].clone();
    // A long word may break across two lines, so the marked pieces are put back together.
    let words = |layout: &folio_layout::wire::SpreadLayout| -> String {
        layout
            .frames
            .iter()
            .flat_map(|f| f.lines.iter())
            .flat_map(|l| l.runs.iter())
            .filter(|r| !r.miss.is_empty())
            .map(|r| r.text.clone())
            .collect()
    };
    // Put a made-up word into the spread's first text frame.
    let frame = spread
        .frames
        .iter()
        .find(|f| matches!(f, folio_model::Frame::Text(_)))
        .unwrap()
        .base()
        .id
        .clone();
    books(&app)
        .apply(
            &id,
            Intent::SetText {
                frame,
                text: "A small otter named Zorblat swam by.".into(),
            },
        )
        .unwrap();
    let laid = |spelling: Option<&[String]>| {
        books(&app)
            .layout_spread(&id, &spread.id.0, spelling)
            .unwrap()
            .layout
    };
    assert!(words(&laid(None)).is_empty(), "off unless asked");
    assert_eq!(words(&laid(Some(&[]))), "Zorblat");
    // Ignored in the book, or added for every book, it is no longer marked.
    assert!(words(&laid(Some(&["Zorblat".to_owned()]))).is_empty());
    books(&app)
        .apply(
            &id,
            Intent::IgnoreWord {
                word: "Zorblat".into(),
            },
        )
        .unwrap();
    assert!(words(&laid(Some(&[]))).is_empty());
}

#[test]
fn the_proofing_list_finds_misspelt_words_in_frames_cells_and_flowing_text_with_where_they_are() {
    let app = app();
    let snapshot = books(&app).new_book(&NewBookSpec::default()).unwrap();
    let id = snapshot.summary.id.clone();
    let spread = snapshot.book.spreads[1].clone();
    let on = folio_model::Surface::Spread(spread.id.clone());
    let text_frame = spread
        .frames
        .iter()
        .find(|f| matches!(f, folio_model::Frame::Text(_)))
        .unwrap()
        .base()
        .id
        .clone();
    books(&app)
        .apply(
            &id,
            Intent::SetText {
                frame: text_frame.clone(),
                text: "A small ottr named Pip met a zorb.".into(),
            },
        )
        .unwrap();
    let table: folio_model::Frame = serde_json::from_value(serde_json::json!({
        "type": "table", "id": "fr_table", "box": { "x": 10, "y": 10, "w": 200, "h": 60 },
        "cols": [0.5, 0.5],
        "rows": [
            [{ "content": { "inline": [{ "text": "fine" }] } }, { "content": { "inline": [{ "text": "blorp" }] } }],
            [{ "content": { "inline": [{ "text": "bold", "bold": true }, { "text": " wrng" }] } }, {}]
        ]
    }))
    .unwrap();
    books(&app)
        .apply(&id, Intent::AddFrame { on, frame: table })
        .unwrap();
    let found = books(&app).spell_check_book(&id, &[]).unwrap();
    assert!(found.supported && !found.truncated);
    let words: Vec<_> = found
        .words
        .iter()
        .map(|w| {
            (
                w.word.as_str(),
                w.frame.as_str(),
                w.row,
                w.col,
                w.replaceable,
            )
        })
        .collect();
    let first = text_frame.0.as_str();
    assert!(
        words.contains(&("ottr", first, None, None, true)),
        "{words:?}"
    );
    assert!(
        words.contains(&("zorb", first, None, None, true)),
        "{words:?}"
    );
    assert!(
        words.contains(&("blorp", "fr_table", Some(0), Some(1), true)),
        "{words:?}"
    );
    // Words with emphasis are found, and are changed on the page.
    assert!(
        words.contains(&("wrng", "fr_table", Some(1), Some(0), false)),
        "{words:?}"
    );
    let ottr = found.words.iter().find(|w| w.word == "ottr").unwrap();
    assert_eq!((ottr.start, ottr.end), (8, 12));
    assert!(
        ottr.context.contains("small ottr named"),
        "{}",
        ottr.context
    );
    assert_eq!(ottr.spread, spread.id.0);
    // What the book ignores, or the person adds for every book, is not listed.
    books(&app)
        .apply(
            &id,
            Intent::IgnoreWord {
                word: "zorb".into(),
            },
        )
        .unwrap();
    let after = books(&app)
        .spell_check_book(&id, &["blorp".to_owned()])
        .unwrap();
    let left: Vec<_> = after.words.iter().map(|w| w.word.as_str()).collect();
    assert!(!left.contains(&"zorb") && !left.contains(&"blorp") && left.contains(&"ottr"));
}

/// A new app whose recovery copies are kept in `dir`, as a launch after a crash would find them.
fn app_keeping_copies_in(dir: &std::path::Path) -> App {
    let app = app();
    books(&app).recovery.keep_in(dir.to_path_buf());
    app
}

#[test]
fn a_book_with_changes_is_copied_once_for_each_change_and_never_before_it_is_touched() {
    let dir = tempfile::tempdir().unwrap();
    let app = app_keeping_copies_in(dir.path());
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    // Starting a book and leaving it alone costs nothing.
    assert_eq!(books(&app).autosave(), 0);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);

    books(&app).apply(&id, nudge(&app, &id, 10.0)).unwrap();
    assert_eq!(books(&app).autosave(), 1);
    // Nothing has changed since, so nothing is written again, until something does.
    assert_eq!(books(&app).autosave(), 0);
    books(&app).apply(&id, nudge(&app, &id, 20.0)).unwrap();
    assert_eq!(books(&app).autosave(), 1);

    // The book that is open is not offered back to itself.
    assert!(books(&app).recoverable().is_empty());
}

#[test]
fn a_book_a_crash_left_changes_in_comes_back_unsaved_and_a_save_clears_the_copy() {
    let dir = tempfile::tempdir().unwrap();
    let saved_as = dir.path().join("Books").join("Pip.folio");
    {
        let app = app_keeping_copies_in(&dir.path().join("recovery"));
        let id = books(&app)
            .new_book(&NewBookSpec::default())
            .unwrap()
            .summary
            .id;
        books(&app).save(&id, Some(&saved_as)).unwrap();
        books(&app).apply(&id, nudge(&app, &id, 40.0)).unwrap();
        assert_eq!(books(&app).autosave(), 1);
        // The app stops here without closing the book: a crash, or the power going.
    }

    let app = app_keeping_copies_in(&dir.path().join("recovery"));
    let found = books(&app).recoverable();
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].path.as_deref(),
        Some(saved_as.canonicalize().unwrap().to_string_lossy().as_ref())
    );

    let restored = books(&app).restore(&found[0].id).unwrap();
    // As it was: the same file, still unsaved, and what was changed is in it.
    assert_eq!(restored.summary.path, found[0].path);
    assert!(restored.summary.dirty);
    let cover = &restored.book.spreads[0];
    assert_eq!(cover.frames[0].base().rect.x.0, 40.0);
    // An open book is not offered again, and cannot be thrown away from under itself.
    assert!(books(&app).recoverable().is_empty());
    books(&app).discard(&found[0].id);
    assert_eq!(
        std::fs::read_dir(dir.path().join("recovery"))
            .unwrap()
            .count(),
        1
    );

    // Saving writes the real file and takes the copy away.
    let saved = books(&app).save(&restored.summary.id, None).unwrap();
    assert!(!saved.dirty);
    assert_eq!(
        std::fs::read_dir(dir.path().join("recovery"))
            .unwrap()
            .count(),
        0
    );
    let reopened = Package::open(&saved_as).unwrap();
    assert_eq!(reopened.book().spreads[0].frames[0].base().rect.x.0, 40.0);
}

#[test]
fn a_book_never_saved_comes_back_untitled_and_closing_or_giving_up_leaves_no_copy() {
    let dir = tempfile::tempdir().unwrap();
    {
        let app = app_keeping_copies_in(dir.path());
        let id = books(&app)
            .new_book(&NewBookSpec::default())
            .unwrap()
            .summary
            .id;
        books(&app).apply(&id, nudge(&app, &id, 55.0)).unwrap();
        books(&app).autosave();
    }
    let app = app_keeping_copies_in(dir.path());
    let found = books(&app).recoverable();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, None);

    // Restored, it has changes worth asking about even though none was made this time.
    let restored = books(&app).restore(&found[0].id).unwrap();
    assert!(restored.summary.dirty && restored.summary.path.is_none());
    assert!(restored.revision > 0);

    // Closing it on purpose leaves no copy behind.
    books(&app).close(&restored.summary.id).unwrap();
    assert!(books(&app).recoverable().is_empty());

    // Giving one up works too, and an id that is not one of ours names nothing.
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    books(&app).apply(&id, nudge(&app, &id, 5.0)).unwrap();
    books(&app).autosave();
    let other = app_keeping_copies_in(dir.path());
    let theirs = books(&other).recoverable();
    assert_eq!(theirs.len(), 1);
    books(&other).discard("../outside");
    books(&other).discard(&theirs[0].id);
    assert!(books(&other).recoverable().is_empty());
    assert!(matches!(
        books(&other).restore("../outside"),
        Err(Error::Recovery(_))
    ));
}

#[test]
fn a_book_offers_the_bundled_families_first_and_says_where_each_comes_from() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let choices = books(&app).font_choices(&id).unwrap();
    assert_eq!(choices[0].origin, crate::wire::FontOrigin::Bundled);
    assert!(choices.iter().any(|c| c.family == "Literata"));
    let mut names: Vec<String> = choices.iter().map(|c| c.family.to_lowercase()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), choices.len(), "each family once");
    assert!(books(&app).font_choices("nope").is_err());
}

#[test]
fn keeping_fonts_leaves_the_bundled_ones_alone_and_reports_what_is_not_installed() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let report = books(&app)
        .keep_fonts(&id, &["Literata".to_string(), "No Such Family".to_string()])
        .unwrap();
    assert!(report.kept.is_empty());
    assert_eq!(report.skipped.len(), 1);
    assert_eq!(report.skipped[0].family, "No Such Family");
    assert_eq!(
        report.skipped[0].reason,
        crate::wire::FontSkip::NotInstalled
    );
    // Nothing was kept, so nothing changed and there is nothing to undo.
    assert!(books(&app).snapshot(&id).unwrap().book.fonts.is_empty());
    assert!(!books(&app).snapshot(&id).unwrap().can_undo);
}

#[test]
fn an_installed_font_that_may_travel_is_kept_in_one_undo_step_and_laid_out_from_the_book() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    // Which fonts are installed depends on the computer, so use the first one whose licence allows.
    let installed: Vec<String> = books(&app)
        .font_choices(&id)
        .unwrap()
        .into_iter()
        .filter(|c| c.origin == crate::wire::FontOrigin::System)
        .map(|c| c.family)
        .collect();
    let Some(family) = installed.into_iter().find(|f| {
        let report = books(&app)
            .keep_fonts(&id, std::slice::from_ref(f))
            .unwrap();
        !report.kept.is_empty()
    }) else {
        return; // No installed font may be kept on this computer; there is nothing to check.
    };
    let snapshot = books(&app).snapshot(&id).unwrap();
    assert!(snapshot
        .book
        .fonts
        .iter()
        .all(|f| f.path.starts_with("assets/fonts/")));
    assert!(snapshot.book.fonts.iter().any(|f| f.family == family));
    assert!(snapshot.can_undo);
    // The family now comes from the book.
    let origin = books(&app)
        .font_choices(&id)
        .unwrap()
        .into_iter()
        .find(|c| c.family == family)
        .unwrap()
        .origin;
    assert_eq!(origin, crate::wire::FontOrigin::Book);
    // Keeping it again changes nothing.
    let depth = books(&app).history_mark(&id).unwrap();
    books(&app)
        .keep_fonts(&id, std::slice::from_ref(&family))
        .unwrap();
    assert_eq!(books(&app).history_mark(&id).unwrap(), depth);
    // One undo takes it back out.
    books(&app).undo(&id).unwrap();
    assert!(books(&app).snapshot(&id).unwrap().book.fonts.is_empty());
}

#[test]
fn preflight_gathers_the_whole_book_with_what_waits_on_the_person_first() {
    use folio_layout::wire::{PreflightKind, Severity};
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let items = books(&app).preflight(&id).unwrap();
    // A sample picture book has picture frames with no picture yet.
    assert!(items
        .iter()
        .any(|i| i.kind == PreflightKind::PicturePlaceholder));
    let severities: Vec<Severity> = items.iter().map(|i| i.severity).collect();
    let mut sorted = severities.clone();
    sorted.sort();
    assert_eq!(severities, sorted, "errors, then warnings, then notes");
    assert!(books(&app).preflight("nope").is_err());
}

#[test]
fn exporting_writes_each_target_into_an_exports_folder_beside_the_saved_book() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Otter").join("Otter.folio");
    books(&app).save(&id, Some(file.as_path())).unwrap();

    let report = books(&app)
        .export(
            &id,
            &[
                ExportTarget::SharingPdf,
                ExportTarget::KdpInterior,
                ExportTarget::KdpCover,
                ExportTarget::EbookCover,
            ],
            ExportColour::default(),
        )
        .unwrap();
    let folder = dir.path().join("Otter").join("exports");
    assert_eq!(std::path::Path::new(&report.folder), folder);
    assert_eq!(report.files.len(), 4);
    let names: Vec<String> = report
        .files
        .iter()
        .map(|f| {
            std::path::Path::new(&f.path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(
        names,
        [
            "the-otter-who-couldnt-sleep-share.pdf",
            "the-otter-who-couldnt-sleep-paperback-interior.pdf",
            "the-otter-who-couldnt-sleep-paperback-cover.pdf",
            "the-otter-who-couldnt-sleep-ebook-cover.jpg"
        ]
    );
    for file in &report.files {
        let bytes = std::fs::read(&file.path).unwrap();
        assert!(bytes.starts_with(b"%PDF-") || bytes.starts_with(&[0xff, 0xd8]));
        assert_eq!(bytes.len() as u32, file.bytes);
    }
    // No half-written file is left behind.
    assert!(std::fs::read_dir(&folder).unwrap().all(|e| !e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .ends_with(".partial")));
    assert!(books(&app)
        .export("nope", &[ExportTarget::SharingPdf], ExportColour::default())
        .is_err());
}

#[test]
fn the_interior_can_be_made_in_ink_and_a_bad_profile_is_refused() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec {
            title: "Otter".into(),
            ..NewBookSpec::default()
        })
        .unwrap()
        .summary
        .id;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Otter").join("Otter.folio");
    books(&app).save(&id, Some(file.as_path())).unwrap();
    let ink = ExportColour {
        cmyk: true,
        profile: None,
    };
    let report = books(&app)
        .export(&id, &[ExportTarget::KdpInterior], ink)
        .unwrap();
    let in_ink = std::fs::read(&report.files[0].path).unwrap();
    let plain = books(&app)
        .export(&id, &[ExportTarget::KdpInterior], ExportColour::default())
        .unwrap();
    // The colour model is in the page content, so the two files differ.
    assert_ne!(in_ink, std::fs::read(&plain.files[0].path).unwrap());

    let junk = dir.path().join("junk.icc");
    std::fs::write(&junk, b"not a profile").unwrap();
    let bad = ExportColour {
        cmyk: true,
        profile: Some(junk.to_string_lossy().into_owned()),
    };
    assert!(books(&app)
        .export(&id, &[ExportTarget::KdpInterior], bad)
        .is_err());
}

#[test]
fn the_reader_preview_is_the_chapters_and_css_the_epub_ships() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec {
            kind: folio_model::BookKind::Novel,
            ..NewBookSpec::default()
        })
        .unwrap()
        .summary
        .id;
    let preview = books(&app).reader_preview(&id).unwrap();
    assert_eq!(preview.language, "en-CA");
    assert!(!preview.chapters.is_empty());
    assert!(preview.chapters.iter().any(|c| c.body.contains("<p")));
    assert!(preview.css.contains("font-family"));
    assert!(books(&app).reader_preview("nope").is_err());
}

#[test]
fn the_fonts_panel_lists_the_families_a_book_uses_and_a_kept_font_can_be_let_go() {
    use crate::wire::{FontLicence, FontOrigin};
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let in_use = books(&app).fonts_in_use(&id).unwrap();
    let look = in_use
        .iter()
        .find(|f| f.family == "Atkinson Hyperlegible")
        .expect("the look");
    assert_eq!(look.origin, FontOrigin::Bundled);
    assert!(!look.missing);
    assert_eq!(look.licence, FontLicence::Free);

    // A font nobody has is reported missing.
    let snapshot = books(&app).snapshot(&id).unwrap();
    let frame = snapshot.book.spreads[1]
        .frames
        .iter()
        .find(|f| matches!(f, folio_model::Frame::Text(_)))
        .unwrap()
        .base()
        .id
        .clone();
    books(&app)
        .apply(
            &id,
            Intent::OverrideStyle {
                frame,
                overrides: folio_model::StyleOverrides {
                    font: Some("Zed Sans".into()),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let in_use = books(&app).fonts_in_use(&id).unwrap();
    assert!(in_use.iter().any(|f| f.family == "Zed Sans" && f.missing));

    // Letting go of a font that is not kept changes nothing.
    let depth = books(&app).history_mark(&id).unwrap();
    books(&app)
        .unkeep_fonts(&id, &["Zed Sans".to_string()])
        .unwrap();
    assert_eq!(books(&app).history_mark(&id).unwrap(), depth);
}

#[test]
fn an_issue_the_person_leaves_alone_is_marked_so_and_comes_back_when_it_is_let_go() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let all = books(&app).preflight(&id).unwrap();
    assert!(all.iter().all(|i| !i.ignored && !i.key.is_empty()));
    let one = all[0].key.clone();
    books(&app)
        .apply(
            &id,
            Intent::SetPreflightIgnore {
                keys: vec![one.clone()],
            },
        )
        .unwrap();
    let after = books(&app).preflight(&id).unwrap();
    assert_eq!(after.len(), all.len(), "an ignored issue is still listed");
    assert!(after.iter().find(|i| i.key == one).unwrap().ignored);
    assert_eq!(after.iter().filter(|i| i.ignored).count(), 1);
    books(&app).undo(&id).unwrap();
    assert!(books(&app)
        .preflight(&id)
        .unwrap()
        .iter()
        .all(|i| !i.ignored));
}

#[test]
fn a_spread_is_proofed_through_a_press_profile_and_a_bad_path_is_refused() {
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let spread = books(&app).snapshot(&id).unwrap().book.spreads[1]
        .id
        .0
        .clone();
    assert!(books(&app).proof_spread(&id, &spread, "/nope.icc").is_err());
    let press = "/usr/share/ghostscript/iccprofiles/default_cmyk.icc";
    if std::path::Path::new(press).exists() {
        let pages = books(&app).proof_spread(&id, &spread, press).unwrap();
        assert!(!pages.is_empty());
        assert!(pages[0].proofed.starts_with("data:image/png;base64,"));
        assert_ne!(pages[0].original, pages[0].proofed);
    }
}

#[test]
fn an_export_says_which_file_it_has_reached_and_stops_when_asked() {
    use crate::wire::{ExportProgress, ExportReport};
    use crate::EXPORT_PROGRESS_EVENT;
    let app = app();
    let id = books(&app)
        .new_book(&NewBookSpec {
            title: "Otter".into(),
            ..NewBookSpec::default()
        })
        .unwrap()
        .summary
        .id;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Otter").join("Otter.folio");
    books(&app).save(&id, Some(file.as_path())).unwrap();
    let targets = [ExportTarget::SharingPdf, ExportTarget::EbookCover];

    let heard = hear::<ExportProgress>(&app, "main", EXPORT_PROGRESS_EVENT);
    let report: ExportReport = books(&app)
        .export(&id, &targets, ExportColour::default())
        .unwrap();
    assert!(!report.cancelled);
    let seen: Vec<(u32, u32)> = heard
        .lock()
        .unwrap()
        .iter()
        .map(|p| (p.done, p.total))
        .collect();
    assert_eq!(seen, vec![(0, 2), (1, 2)]);

    // Asking to stop when the first file is reached keeps that file and makes no more.
    let handle = app.handle().clone();
    app.listen(EXPORT_PROGRESS_EVENT, move |e| {
        let p: ExportProgress = serde_json::from_str(e.payload()).unwrap();
        if p.done == 1 {
            handle.state::<Books<MockRuntime>>().cancel_export();
        }
    });
    let stopped = books(&app)
        .export(&id, &targets, ExportColour::default())
        .unwrap();
    assert!(stopped.cancelled);
    assert_eq!(stopped.files.len(), 1);
}

#[test]
fn the_cover_a_new_book_would_have_is_drawn_without_making_the_book() {
    let app = app();
    let spec = NewBookSpec {
        cover_colour: "#1f4e5f".into(),
        ..NewBookSpec::default()
    };
    let uri = books(&app).preview_new_book(&spec).unwrap();
    assert!(uri.starts_with("data:image/png;base64,"));
    assert!(books(&app).list().is_empty());
    let other = books(&app)
        .preview_new_book(&NewBookSpec {
            cover_colour: "#a23b2a".into(),
            ..spec
        })
        .unwrap();
    assert_ne!(uri, other);
    assert!(books(&app)
        .preview_new_book(&NewBookSpec {
            cover_colour: "teal".into(),
            ..NewBookSpec::default()
        })
        .is_err());
}

#[test]
fn a_markdown_file_becomes_the_books_story_in_one_undo_step_and_a_book_without_one_says_so() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("story.md");
    std::fs::write(
        &file,
        "# Chapter one\n\nThe otter slept.\n\n***\n\nMorning came.\n",
    )
    .unwrap();
    let path = file.to_string_lossy().into_owned();

    let novel = books(&app)
        .new_book(&NewBookSpec {
            kind: folio_model::BookKind::Novel,
            ..NewBookSpec::default()
        })
        .unwrap()
        .summary
        .id;
    let report = books(&app).import_text(&novel, &path).unwrap();
    assert_eq!(
        (report.paragraphs, report.headings, report.scene_breaks),
        (4, 1, 1)
    );
    let story = |id: &str| -> Vec<String> {
        books(&app).snapshot(id).unwrap().book.flows[0]
            .paragraphs
            .iter()
            .map(|p| p.text())
            .collect()
    };
    assert_eq!(story(&novel)[0], "Chapter one");
    assert_eq!(story(&novel).len(), 4);
    books(&app).undo(&novel).unwrap();
    assert!(story(&novel)[0].starts_with("Mara had kept"));

    // A picture book has no flow, and a file that is not text is refused in plain words.
    let picture = books(&app)
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id;
    let refused = books(&app).import_text(&picture, &path).unwrap_err();
    assert!(refused.to_string().contains("no story"), "{refused}");
    let doc = dir.path().join("story.docx");
    std::fs::write(&doc, b"PK").unwrap();
    assert!(books(&app)
        .import_text(&novel, &doc.to_string_lossy())
        .unwrap_err()
        .to_string()
        .contains("Markdown"));
}
