# Decisions

Product and engineering decisions for Folio. D-01 to D-07 and D-12 are defined where they arise in `spec/` and are listed here so they can be found in one place. New decisions are added here first.

| ID   | Decision                                                                                                                                             | Where                        |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------- |
| D-01 | Easy and Expert modes; chosen on the welcome screen, remembered per user, never removes data                                                         | `spec/01-product.md`         |
| D-02 | The document model lives in Rust and is the source of truth; the UI sends intents and receives patches                                               | `spec/02-architecture.md`    |
| D-03 | Window chrome comes from the shared Liminal chrome package (title bar, app menu button, window menu, context menu)                                   | `spec/02-architecture.md`    |
| D-04 | Theming follows the system colour scheme and accent; book pages are never themed. Tokens are Folio's own (`--folio-*`, `data-folio-theme`), see D-09 | `spec/02-architecture.md`    |
| D-05 | `.folio` schema is an integer with migrations on open; files are never rewritten silently                                                            | `spec/03-project-format.md`  |
| D-06 | App menu in the title bar's start slot; menu bar hidden by default                                                                                   | `spec/06-ui.md`              |
| D-07 | Ribbon and classic toolbar both ship; the user picks                                                                                                 | `spec/06-ui.md`              |
| D-12 | Grouped sidebar, tear-off panels, document tabs as windows                                                                                           | `spec/12-sidebar-windows.md` |
| D-20 | Layout is Rust-owned and drawn by the UI from lines Rust positions                                                                                   | `docs/decisions.md`          |
| D-19 | The ribbon is its own package, depending one-way on the chrome                                                                                       | `docs/decisions.md`          |
| D-21 | Ruler guides belong to a spread, are stored in points, and are named by their place in its list                                                      | `docs/decisions.md`          |
| D-22 | Masters belong to the person, and frames and guides are addressed by a surface                                                                       | `docs/decisions.md`          |
| D-23 | A page's background is a field on the page, its spread or its master, and the nearest wins                                                           | `docs/decisions.md`          |
| D-24 | Shapes are frames with their own look, and the schema stays additive until the first release                                                         | `docs/decisions.md`          |
| D-25 | Spelling is Folio's own, from a pure-Rust checker and bundled dictionaries, on by default                                                            | `docs/decisions.md`          |
| D-26 | A section is a field on the spread that starts it, and page labels are counted from the sections                                                     | `docs/decisions.md`          |

## D-08 · Shared chrome and unpublished plugins are copied; published ones are dependencies

`packages/chrome` is copied from Waypoint's `packages/chrome` and the unpublished reusable plugins `window-tearoff` and `window-effects` are copied from Waypoint's `plugins/`. Each copy carries a `PROVENANCE.md` with its source repository and commit. Plugins already published to crates.io and npm (`system-appearance`, `os-prefs`, `window-manager`) are registry dependencies, never copies. When a copied package is published, the copy is deleted and replaced with a dependency.

## D-09 · Folio's own design-token names

The copied chrome used Waypoint's `--wp-*` custom properties and `data-wp-theme` attribute. In Folio they are `--folio-*` and `data-folio-theme`, and the package is `@liminal-hq/folio-chrome`. Waypoint's names appear only in `PROVENANCE.md` and in the reference prototype under `design/`. Book paper and ink are separate, never-themed tokens (`--folio-paper`, `--folio-ink`).

## D-10 · App identity

Bundle identifier `ca.liminalhq.folio`, which is also the name of the app's data and log directories. Product name Folio, publisher Liminal HQ, licence `Apache-2.0 OR MIT`.

## D-11 · Settings are Rust-owned and never in browser storage

App-wide preferences (mode, theme, units, toolbar and sidebar layout, window geometry, recent books) live in a typed, versioned settings document owned by the `folio-settings` plugin, with a previous generation kept for recovery. `localStorage`, `sessionStorage` and IndexedDB are not used for app state. Per-book settings stay inside the `.folio` file.

## D-13 · Windows are created in Rust and are shadowed

`tauri.conf.json` keeps `"windows": []`. A `windows` module builds every window with `.shadow(true)`. Windows 11 draws the shadow and rounded corners; Tauri's shadow is unsupported on Linux, so the chrome draws one in CSS inside a small transparent margin.

## D-14 · Dev-only automation

`tauri-plugin-mcp-bridge` is registered only under `#[cfg(all(debug_assertions, not(windows)))]`, `withGlobalTauri` is enabled only by `tauri.conf.dev.json` (merged by `bun run tauri:dev`), and the `dev-automation` capability applies only on Linux and macOS. Release builds contain none of it.

## D-15 · Documentation site

The public documentation and manual are built with Astro Starlight and deployed to GitHub Pages, with a hand-built landing page.

## D-16 · Books are sessions in Rust, and windows are views onto them

Every open book is a `folio-session` over its `folio-package`, held by the `folio-books` plugin. A window asks for a book, sends intents, and applies the patches and events it hears; nothing but Rust changes a book. Any window can show any open book, and undo is shared per book (spec 12).

## D-17 · A new book lives in memory until it is first saved

New writes nothing. Save and Save as… ask for a place, starting at `~/Books/<Title>/<Title>.folio` (`Documents\Books` on Windows). A book never saved asks about its changes on close only once it has been touched, so starting a book and closing it again costs nothing.

## D-18 · Presets and sample pages are Rust-owned data

The six kinds, their sizes, styles and sample spreads are data in `crates/folio-presets`, written once from the design prototype and owned there since. The UI asks Rust for them and keeps no copy. `NewBookSpec` carries every field the New book wizard asks for, so the wizard window is only a screen over it.

## D-20 · Layout is Rust-owned, and the canvas draws what Rust positions

`folio-layout` shapes text (`rustybuzz`), breaks it into lines (Knuth–Plass, first-fit while typing), hyphenates (Liang patterns), pours it through frames and columns (widows and orphans, baseline grid, overset) and checks print geometry. The UI never lays text out: it asks the books plugin for `layout_spread(id, spread)` and draws the lines, with the revision the layout came from, so a stale answer is never drawn over a newer book. The four bundled families (Literata, Source Serif 4, Inter, Atkinson Hyperlegible, all OFL) are always available, so text breaks the same on every machine; installed fonts are offered as well. Dictionaries for hyphenation are embedded (en-US for Canadian English, en-GB, French).

## D-19 · The ribbon is its own package

`packages/ribbon` (`@liminal-hq/folio-ribbon`) holds the ribbon and nothing else: the chrome stays about windowing, and the ribbon knows nothing about books. Folio supplies a `RibbonModel` and handles `onCommand`. The ribbon depends one-way on the chrome for its dropdowns, so menus look and behave the same everywhere. Another Liminal app can take it without taking Folio.

## D-21 · Ruler guides belong to a spread, are stored in points, and are named by their place in its list

A guide is `{ axis, pos, locked }` on its spread, with `pos` in points like every other stored length, whatever unit the rulers show. It has no id: the intents `MoveGuide`, `LockGuide` and `RemoveGuide` name it by its index, which is safe because a guide's list changes only through those intents and each is one undo step. Rulers and guides are shown by settings (`rulers`, `marks.guides`), not by the book, so a book opens the way its reader left the rulers. A guide placed by dragging snaps to margins, frame edges and other guides, else to the unit's fine step (spec 04), and a locked guide refuses every change. Guides that belong to every page live on a master (D-22).

## D-22 · Masters belong to the person, and frames and guides are addressed by a surface

Page numbers used to be the only thing on a master, so the session rebuilt masters A and B whenever pages came, went or moved. A master is now the person's: the page-numbers setting manages only its own two frames (`fr_pn_a`, `fr_pn_b`), by box and alignment, and never deletes a master that holds anything else. A master is made when something first needs one (editing it, putting a guide on every page), and not before, so a book that never uses masters is unchanged. Editing is a separate master view on the stage (Expert), not editing in place, so a click on a page never means "change every page". Intents that name a frame or a guide take a `Surface`, either a spread or a master, instead of a bare spread, and a master edit is undone and patched through the masters the session already captures. Every-page guides are `Master.guides`, in the page's own points, named by their place in the list as D-21 says; "Put on every page" moves a spread guide onto the masters its pages use, converting x to the page's own coordinates and mirroring it for the facing master, and "Only on this spread" moves it back.

## D-23 · A page's background is a field on the page, its spread or its master, and the nearest wins

The prototype kept backgrounds in a parallel array on the spread (`bg[i]`), which drifts from `pages` whenever one moves. A background is now a field of the thing it fills: `Page.bg` for one page, `Spread.bg_span` for one background through the gutter, and `Master.bg` for every page using a master. A page shows its own, else its spread's, else its master's, else the paper. All three sit on spreads and masters, so undo, redo and patches already cover them. `Background` is one of `colour`, `gradient` (an angle and explicit stops, so a book never depends on the palette it was made with), `texture` (a bundled id, optionally tinted) or `picture` (an asset, `fill` or `fit`, and a focus). A background is part of the page, not a frame: it reaches into the bleed on the outer edge, cannot be selected or moved, and is drawn behind every frame. Rust resolves the winner (`resolve`), the canvas and the shelf thumbnails use a twin written against one shared fixture, and preflight counts a background as bleeding. The three fields are optional and left out when empty, so a book without backgrounds is unchanged and the schema stays at 1. A new book's cover colour is the cover page's own background, not a full-bleed shape.

## D-24 · Shapes are frames with their own look, and the schema stays additive until the first release

A shape is one frame, not a path: a `Shape` kind (`rect`, `round`, `ellipse`, `line`, `speech`, `thought`, `shout`) with a box, a `fill`, an optional `stroke` and `strokeWidth`, a `radius` for `round` and a `flip` for a line. There is no polygon or path model, so a shape cannot be edited point by point; each kind has a unit path the app scales to the box and draws with a non-scaling stroke, so an outline keeps its width as the frame is resized. A line is a box that may have no height or width. The note box is a `round` shape with a yellow fill. Words in a shape are fields on the shape (`content`, `style`, `overrides`), not a child frame in a group: `folio-layout` lays them into the kind's body with the `bubble` or `callout` style, so every frame operation (move, resize, lock, undo) treats shape and words as one. A frame turns about the middle of its box (`rotation` in degrees clockwise, 0–360); snapping and preflight use the upright box around the turned corners, so the one rule serves every frame type. Each is optional so an older book opens unchanged. A table is likewise one frame (`Frame::Table`): column shares, rows of cells holding inline words, a header-row count and a look, with no catch-all for unknown frame types because no book predates it. Its shape is checked when it is added (1–50 rows and columns, rectangular, positive shares adding up to 1), and layout sets each cell in the `cell` style in its own room, named `<table>#<row>.<column>`.

Until the first public release (the version is 0.0.0 and nothing is tagged), the schema stays at 1 and changes are additive: a new optional field or a new frame or shape kind is added without a migration. After it, a new kind of frame or shape bumps the schema with a migration (D-05), since a build that does not know the kind cannot open the book.

## D-26 · A section is a field on the spread that starts it, and page labels are counted from the sections

A book's page numbers change style or start again in places: Roman front matter, then Arabic from 1. A section is `{ name, numbering: arabic | roman | none, start? }` on the spread that begins it, and runs to the next spread that begins one. It is a field on the spread, not a list of indices on the book, so moving, deleting or duplicating spreads cannot leave a section pointing at the wrong place, and undo and patches (`SetSpread`) carry it with no new machinery. A duplicate does not copy the section, and the cover cannot start one. Pages before the first section are in an implied Arabic section counted from 1. Counting carries on across sections unless `start` restarts it, and a section with `numbering: none` still counts its pages. `page_labels` in `folio-model` gives every page its written number and its section's name; the canvas's `pageLabels` is its twin, and both run the cases in `labelCases.json`. The `{page}` token takes the written number, so a Roman page shows `iii`, and the new `{section}` token takes the section's name. A section's own master, so that front matter can carry different furniture, is not part of this step: masters are still given by side, and page numbers are added to any master only when the person asks.

## D-25 · Spelling is Folio's own, from a pure-Rust checker and bundled dictionaries, on by default

The webview's checker cannot give a book a spelling language: WebView2 has no setting for it, and WebKitGTK's is global and also changes `navigator.language` and the `Intl` default. Neither can check text Rust owns, such as a shape's words or a table's cells, or a run in another language. So Folio checks spelling itself. `folio-spell` is a pure-Rust crate over a small `Checker` trait, so the engine can be swapped; the first engine is `spellbook`, a Hunspell-compatible checker reading `.aff` and `.dic` files, and the first dictionary is the en-CA one from `wooorm/dictionaries`, bundled, with its SCOWL notice kept beside it. Other languages are downloaded when the person asks, each pinned by version and SHA-256 in a manifest in the repo (`crates/folio-spell/data/dictionaries.json`, fetched from the npm package through a CDN that serves its files verbatim), and each keeps its own licence beside it. A download is checked file by file and kept only whole. The catalogue holds the dictionaries under permissive or weak-copyleft licences (MIT/BSD, MPL, LGPL, CC-BY); GPL-only ones (German, Italian, Swedish) are left out until that is decided. `spellbook` is MPL-2.0, which is file-level copyleft: it is used unmodified, the licence check allows MPL-2.0 for that one crate and no other, and its notice ships in the third-party notices. The webview's own underline is turned off on the editing overlay, and the app draws underlines from ranges Rust returns, with suggestions on request. Checking is on by default (a book is words, and a misspelling in a printed book is permanent), and `spelling.enabled` in the settings document turns it off. Words the person ignores are kept per book in `proofing.ignore`, so they travel with the file and undo with it, and a personal list for every book (`spelling.personal` in the settings document) is added to and trimmed from the editor's menu and Preferences.
