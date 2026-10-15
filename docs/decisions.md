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
| D-23 | A page's background is a field on the page, its spread or its master, and the page that is closest wins                                              | `docs/decisions.md`          |

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

The prototype kept backgrounds in a parallel array on the spread (`bg[i]`), which drifts from `pages` whenever one moves. A background is now a field of the thing it fills: `Page.bg` for one page, `Spread.bg_span` for one background through the gutter, and `Master.bg` for every page using a master. A page shows its own, else its spread's, else its master's, else the paper. All three sit on spreads and masters, so undo, redo and patches already cover them. `Background` is one of `colour`, `gradient` (an angle and explicit stops, so a book never depends on the palette it was made with), `texture` (a bundled id, optionally tinted) or `picture` (an asset, `fill` or `fit`, and a focus). A background is part of the page, not a frame: it reaches into the bleed on the outer edge, cannot be selected or moved, and is drawn behind every frame. Rust resolves the winner (`resolve`), the canvas and the shelf thumbnails use a twin written against one shared fixture, and preflight counts a background as bleeding. The three fields are optional and left out when empty, so a book without backgrounds is unchanged and the schema stays at 1.
