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
        books(&app).spelling.download("../../etc"),
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
    books(&app).spelling.download("fr").unwrap();
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
