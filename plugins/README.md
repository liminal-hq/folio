# Plugins

Tauri plugins used by Folio. Each is a Rust crate plus a `guest-js` package, and the app reaches it only through `guest-js` (`scripts/check-plugin-boundaries.sh`).

## Two tiers

- **Reusable plugins** (`tauri-plugin-{name}`) are generic and never mention Folio, so they can graduate to the shared `liminal-hq/tauri-plugins-workspace` unchanged.
- **Domain plugins** (`tauri-plugin-folio-{name}`) are Folio's own: they adapt a pure `crates/folio-*` crate to the Tauri surface.

## Published plugins are dependencies, not copies

`tauri-plugin-system-appearance`, `tauri-plugin-os-prefs` and `tauri-plugin-window-manager` (and their `@liminal-hq/plugin-*` guest packages) are published, so the app depends on the registry releases and nothing is kept here. Only a plugin that is not yet published is copied into this folder, with a `PROVENANCE.md` naming its source and commit. When it is published, delete the copy and depend on the release (decision D-08).

| Plugin           | Tier     | Source               | Status                                      |
| ---------------- | -------- | -------------------- | ------------------------------------------- |
| `window-tearoff` | reusable | Copied from Waypoint | Built; registered when document tabs arrive |
| `window-effects` | reusable | Copied from Waypoint | Built; registered in the app shell          |
| `folio-settings` | domain   | Folio                | Typed, versioned app settings (D-11)        |

## Rules

- A reusable plugin imports nothing from Folio and does not mention it.
- Plugins never call each other; the app composes them.
- Every command reachable from a webview has a `permissions/default.toml` entry, and every plugin reports availability so the UI can hide what does not work on the current system.

`shared/rollup.config.js` is also copied from Waypoint (the Rollup configuration every plugin's `guest-js` build uses).
