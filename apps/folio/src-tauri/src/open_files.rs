// Receives the `.folio` files the system asks Folio to open, on a cold start and from a second launch
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Double-clicking a book in the file manager starts `folio <path>`. The first launch reads its own
// arguments; a later launch is stopped by the single-instance plugin and its arguments are handed to
// the running app instead. Either way the paths wait here until the main window takes them, so one
// that arrives before the page has loaded is not lost, and the window opens each through the same
// path as File → Open (which asks about a crash's recovery copy and reports a bad file).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use folio_protocol::WindowKind;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::windows;

/// Sent to the main window when paths are waiting for it.
pub const OPEN_FILES_EVENT: &str = "folio://open-files";

/// The paths asked for and not yet taken by the main window.
#[derive(Default)]
pub struct PendingOpens(Mutex<Vec<String>>);

/// The `.folio` paths among a launch's arguments, in order. The first argument is the program.
///
/// A relative path is read from `cwd`, the directory the launch came from. The Linux desktop entry
/// passes `file://` addresses (`%U`), which are turned back into paths. A path is not checked to
/// exist: opening it reports a missing or unreadable file, which is better than ignoring the click.
pub fn book_paths<I, S>(args: I, cwd: Option<&Path>) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .skip(1)
        .filter_map(|arg| {
            let arg = arg.as_ref();
            let path = match arg.strip_prefix("file://") {
                Some(rest) => from_file_address(rest),
                None => arg.to_owned(),
            };
            if !path.to_ascii_lowercase().ends_with(".folio") {
                return None;
            }
            let path = PathBuf::from(path);
            let path = match cwd {
                Some(cwd) if path.is_relative() => cwd.join(path),
                _ => path,
            };
            Some(path.to_string_lossy().into_owned())
        })
        .collect()
}

/// The path in a `file://` address, after the `file://`: percent escapes decoded, and the slash
/// before a Windows drive letter dropped (`/C:/books` is `C:/books`).
fn from_file_address(rest: &str) -> String {
    // `file://host/path` carries a host before the path; only the local one is meant.
    let rest = rest.find('/').map_or("", |slash| &rest[slash..]);
    let bytes = rest.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let escaped = (bytes[i] == b'%' && i + 2 < bytes.len())
            .then(|| std::str::from_utf8(&bytes[i + 1..i + 3]).ok())
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match escaped {
            Some(byte) => {
                decoded.push(byte);
                i += 3;
            }
            None => {
                decoded.push(bytes[i]);
                i += 1;
            }
        }
    }
    let text = String::from_utf8_lossy(&decoded).into_owned();
    let drive = text.as_bytes();
    if drive.len() >= 3 && drive[0] == b'/' && drive[1].is_ascii_alphabetic() && drive[2] == b':' {
        text[1..].to_owned()
    } else {
        text
    }
}

/// Keeps `paths` for the main window. With `announce` set the window is told, and brought forward;
/// at start-up it is not built yet and will ask for them itself.
pub fn queue<R: Runtime>(app: &AppHandle<R>, paths: Vec<String>, announce: bool) {
    if paths.is_empty() {
        return;
    }
    if let Ok(mut pending) = app.state::<PendingOpens>().0.lock() {
        pending.extend(paths);
    }
    if !announce {
        return;
    }
    let app = app.clone();
    // Building a window waits on the main thread, and the single-instance callback is not on it, so
    // the work moves to the async runtime, as `open_window` does.
    tauri::async_runtime::spawn(async move {
        match windows::open_or_focus(&app, WindowKind::Main) {
            Ok(window) => {
                let _ = app.emit_to(window.label(), OPEN_FILES_EVENT, ());
            }
            Err(error) => log::warn!("could not show the main window for a book: {error}"),
        }
    });
}

/// Hands the main window every path waiting for it, and forgets them.
#[tauri::command]
pub async fn take_open_files<R: Runtime>(app: AppHandle<R>) -> Vec<String> {
    app.state::<PendingOpens>()
        .0
        .lock()
        .map(|mut pending| std::mem::take(&mut *pending))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_folio_files_are_taken_and_the_program_is_skipped() {
        let args = [
            "C:\\Folio\\folio.exe",
            "--flag",
            "C:\\My Books\\Naïve café.folio",
            "notes.txt",
        ];
        assert_eq!(
            book_paths(args, None),
            vec!["C:\\My Books\\Naïve café.folio".to_owned()]
        );
        // The program itself is never read as a book, even when it is named like one.
        assert!(book_paths(["a.folio"], None).is_empty());
    }

    #[test]
    fn several_books_keep_their_order_and_the_extension_ignores_case() {
        let args = ["folio", "/b/one.folio", "/b/two.FOLIO"];
        assert_eq!(book_paths(args, None), vec!["/b/one.folio", "/b/two.FOLIO"]);
    }

    #[test]
    fn a_relative_path_is_read_from_where_the_launch_came_from() {
        let found = book_paths(["folio", "story.folio"], Some(Path::new("/home/me")));
        assert_eq!(
            found,
            vec![Path::new("/home/me").join("story.folio").to_string_lossy()]
        );
    }

    #[test]
    fn a_file_address_becomes_a_path() {
        assert_eq!(
            book_paths(["folio", "file:///home/me/My%20Book%20%C3%A9.folio"], None),
            vec!["/home/me/My Book é.folio"]
        );
        assert_eq!(
            book_paths(["folio", "file:///C:/My%20Books/a.folio"], None),
            vec!["C:/My Books/a.folio"]
        );
    }

    #[test]
    fn a_stray_percent_sign_is_kept() {
        assert_eq!(
            book_paths(["folio", "file:///b/100%.folio"], None),
            vec!["/b/100%.folio"]
        );
    }

    #[test]
    fn queued_paths_are_taken_once() {
        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("the mock app builds");
        app.manage(PendingOpens::default());
        queue(
            app.handle(),
            vec!["a.folio".into(), "b.folio".into()],
            false,
        );
        let taken = tauri::async_runtime::block_on(take_open_files(app.handle().clone()));
        assert_eq!(taken, vec!["a.folio", "b.folio"]);
        assert!(tauri::async_runtime::block_on(take_open_files(app.handle().clone())).is_empty());
    }
}
