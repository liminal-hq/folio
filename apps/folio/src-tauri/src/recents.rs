// Keeps the recent books list: a book that opens or saves goes to the front
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// The books plugin does not know about the settings and the settings plugin does not know about
// books, so the app joins them here: the books plugin announces a book that opened or was saved,
// and the settings record its path.

use folio_settings::push_recent;
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime};

use tauri_plugin_folio_books::{BookEvent, Books};
use tauri_plugin_folio_settings::SettingsStore;

/// Subscribes the recents list to the open books. Call once the plugins are set up.
pub fn wire<R: Runtime>(app: &AppHandle<R>) {
    let Some(books) = app.try_state::<Arc<Books>>() else {
        return;
    };
    let handle = app.clone();
    books.on_book(move |event| {
        let (BookEvent::Opened(summary) | BookEvent::Saved(summary)) = event else {
            return;
        };
        let Some(path) = &summary.path else {
            return;
        };
        let Some(settings) = handle.try_state::<SettingsStore<R>>() else {
            return;
        };
        let recent = push_recent(&settings.get().recent_books, path);
        if let Err(error) = settings.update(&json!({ "recentBooks": recent })) {
            log::warn!("could not record the recent book: {error}");
        }
    });
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use folio_presets::NewBookSpec;
    use folio_settings::MemoryStorage;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    use super::*;

    #[test]
    fn a_saved_book_goes_to_the_front_of_the_recents_once() {
        let app = mock_builder()
            .plugin(tauri_plugin_folio_settings::init(Arc::new(
                MemoryStorage::default(),
            )))
            .plugin(tauri_plugin_folio_books::init())
            .build(mock_context(noop_assets()))
            .expect("the mock app builds");
        wire(app.handle());
        let books = app.state::<Arc<Books>>();
        let settings = app.state::<SettingsStore<tauri::test::MockRuntime>>();
        let dir = tempfile::tempdir().unwrap();

        let first = books.new_book(&NewBookSpec::default()).unwrap().summary.id;
        // A book that has never been saved has no path, so there is nothing to remember yet.
        assert!(settings.get().recent_books.is_empty());
        let one = dir.path().join("one.folio");
        books.save(&first, Some(&one)).unwrap();
        books.save(&first, None).unwrap();
        assert_eq!(
            settings.get().recent_books.len(),
            1,
            "saving twice lists it once"
        );

        let second = books.new_book(&NewBookSpec::default()).unwrap().summary.id;
        let two = dir.path().join("two.folio");
        books.save(&second, Some(&two)).unwrap();
        let recent = settings.get().recent_books;
        assert_eq!(recent.len(), 2);
        assert!(recent[0].ends_with("two.folio") && recent[1].ends_with("one.folio"));
    }
}
