# tauri-plugin-folio-settings

Folio's settings plugin: it holds the application settings as one `Settings` document over the `folio-settings` crate, saves it through a storage the app injects, and tells every window about every change.

This is a domain plugin, private to Folio. Its JavaScript API is the `@liminal-hq/folio-plugin-settings` package in `guest-js/`; the app reaches the plugin only through it, and its wire types come from `@liminal-hq/folio-protocol`. Per-book settings live inside the `.folio` file and never here.

## How it works

- **One document, one writer.** `SettingsStore<R>` (Tauri state) holds the `SettingsSnapshot { revision, settings }`. `update_settings` lays a JSON merge patch over the settings in force, validates the result, saves it, then makes it current, sends the event and runs the change hooks, all under one writer lock, so two changes never interleave. A change that fails validation or cannot be saved changes nothing. A patch that changes nothing is quiet: no revision, no event.
- **Patches, not documents.** A window sends only what it changed. Rust merges it into what is in force now, so a window holding an older copy cannot turn another window's change back.
- **Commands:** `get_status`, `get_settings`, `update_settings` and `reset_settings`. A rejected command is `{ kind: "invalid" | "storage", message }`, plus `field`, `min` and `max` for an out-of-range value, so the page can put the refusal under the row that caused it.
- **Event:** `folio-settings://changed` (`SETTINGS_EVENT`, `onSettingsChanged` in `guest-js`) goes to every window with the new `SettingsSnapshot`. The revision starts at 0 for what was loaded at start-up and grows by one per change; read `getSettings()` first and apply events with a higher revision.
- **Injected storage.** `init(storage)` or `init_with(|app| storage)`, made when the plugin is set up so a file opened through `tauri-plugin-store` can be used. `folio_settings::Persistence` over a `KeyValue` file keeps the document under `settings` with the run before under `settingsPrevious`, and sets a file it cannot read aside as `…corrupt-{unix}`. Settings that cannot be loaded are never fatal: the defaults apply, and an out-of-range stored value is clamped.
- **Change hooks.** The composition root calls `SettingsStore::on_change(|settings| …)` to push what other state depends on without this plugin knowing about it. Hooks run after the new settings are readable and outside the plugin's locks.

## Capabilities

Every window may call all four commands: each window draws the same chrome, and a patch can only change settings, never anything else (`capabilities/settings.json`).
