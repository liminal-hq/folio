# tauri-plugin-folio-books

Folio's books plugin: it holds every open book as a session (`folio-session`) over its package (`folio-package`), and every window sees the same books. Windows are views onto sessions (D-16).

This is a domain plugin, private to Folio. Its JavaScript API is the `@liminal-hq/folio-plugin-books` package in `guest-js/`; the app reaches the plugin only through it, and its wire types come from `@liminal-hq/folio-protocol`.

## How it works

- **One session per open book.** `Books<R>` (Tauri state) holds each `OpenBook` — the session, the package it came from or will be saved to, and its path — behind one lock, so changes never interleave. Opening a path that is already open returns the same session.
- **A new book is in memory until first saved** (D-17). `new_book` writes nothing. `save_book` with no path on a book that was never saved is the typed error `needsPath`; with a path it makes the folder, writes the file atomically and remembers where.
- **Commands:** `get_status`, `presets`, `default_save_path`, `new_book`, `open_book`, `save_book`, `close_book`, `list_books`, `snapshot`, `apply_intent`, `undo`, `redo`, `import_image` (a picture file read into the book, answering its path in the package) and `asset_data` (a picture's bytes), `layout_spread` (one spread laid out: lines of positioned text per frame and the preflight checks, with the revision it was laid out at). A rejected command is `{ kind, message }` with a stable `kind` (see `BooksCommandError` in `guest-js`).
- **Events** go to every window. `folio-books://patch` carries a `PatchEvent` (`id`, the `Patch`, and whether undo and redo are possible) after every apply, undo and redo. `folio-books://changed` carries a `BooksChanged` (the open books as summaries) when books open, close, save or become changed or clean. A window reads `snapshot` first and applies patches with a higher revision.
- **Hooks.** The composition root calls `Books::on_book(|event| …)` to react to a book opening, saving or closing (the recents list) without this plugin knowing about it. Hooks run outside the plugin's locks.
- **Not here:** the autosave journal, import, and the file pickers (the app's dialog plugin owns those).

## Capabilities

Windows that show books may call every command (`capabilities/books.json`).
