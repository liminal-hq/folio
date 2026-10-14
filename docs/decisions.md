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
