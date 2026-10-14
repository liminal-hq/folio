# 02 · Architecture

## Workspace

```
folio/
├─ apps/folio/                 # Tauri app (React + TS, Vite)
│  ├─ src/                     # UI: editor, shelf, inspector, export, welcome, jax
│  └─ src-tauri/               # thin shell: commands, window setup, menus, plugins
├─ crates/
│  ├─ folio-model/             # document model (serde), units, ids, migrations — no IO
│  ├─ folio-package/           # .folio zip read/write, manifest, autosave journal
│  ├─ folio-layout/            # text shaping + line breaking + flow, snapping maths, preflight geometry
│  ├─ folio-export/            # EPUB 3 / KEPUB writer, PDF writer (PDF/X), cover builder
│  ├─ folio-import/            # docx, odt, rtf, md, html-paste, sla, idml, epub → model
│  ├─ folio-color/             # ICC handling (lcms2), RGB→CMYK, output intents
│  └─ folio-jax/               # help library index + ONNX runtime (ort) inference
├─ packages/
│  ├─ protocol/                # ts-rs generated types (from folio-model)
│  └─ chrome/                  # @liminal-hq/folio-chrome (copied from Waypoint): TitleBar, AppMenuButton, MenuBar, ContextMenu, WindowMenu
├─ plugins/
│  ├─ folio-settings/          # domain plugin: typed, versioned app settings (D-11)
│  ├─ window-tearoff/          # copied until published (D-08)
│  └─ window-effects/          # copied until published (D-08)
└─ spec/                       # this folder
```

**D-02:** the model lives in Rust and is the source of truth. The UI holds a mirrored, read-mostly copy and sends **intents** (`MoveFrame`, `SetText`, `AddSpread`…) to Rust, which validates them, applies them, pushes undo entries and emits a patch. This mirrors Waypoint, where Rust owns state.

## Rust crates (key dependencies)

| Crate | Responsibility | Likely deps |
|---|---|---|
| `folio-model` | `Book`, `Spread`, `Page`, `Frame`, `Style`, `Master`, units (`Pt`), id newtypes, schema version + migrations | `serde`, `serde_json`, `ts-rs`, `uuid` |
| `folio-package` | zip container, `mimetype` first + stored, manifest, asset dedupe by SHA-256, autosave journal (`/recovery/`) | `zip`, `sha2`, `tempfile` |
| `folio-layout` | font loading, shaping, Knuth–Plass line breaking with a greedy fallback, hyphenation, frame threading, overset detection, snapping maths shared with the UI via WASM (optional) | `rustybuzz`, `ttf-parser`, `fontdb`, `hyphenation`, `unicode-linebreak` |
| `folio-export` | EPUB 3 (OPF, nav, XHTML, CSS), KEPUB spans, fixed-layout `rendition:layout`, PDF generation, PDF/X output intent, cover PDF | `quick-xml`, `pdf-writer`, `subsetter`, `image` |
| `folio-color` | sRGB/Display-P3 in, CMYK out (GRACoL 2013, FOGRA39), soft-proof previews | `lcms2` |
| `folio-import` | DOCX/ODT (zip + XML), RTF, Markdown (`pulldown-cmark`), HTML paste (`html5ever`), Scribus `.sla` (XML), IDML (zip + XML), EPUB | `quick-xml`, `zip` |
| `folio-jax` | help corpus (Markdown → chunks), embedding index, optional small generative model, guardrails | `ort` (ONNX Runtime), `tokenizers` |

## Tauri commands (first cut)

```
project_new(kind, trim) -> BookSnapshot
project_open(path) -> BookSnapshot           project_save(path?) -> SaveResult
apply_intent(intent) -> Patch                 undo() / redo() -> Patch
layout_spread(spread_id) -> SpreadLayout      // shaped lines for the canvas
reader_preview(target, scale) -> ReaderPages  // reflow preview for Kindle/Kobo
preflight(targets) -> Vec<Issue>
export(targets, options) -> JobId             // progress via `export://progress` events
import_text(path, options) -> ImportReport    import_layout(path) -> ImportReport
jax_ask(question, context) -> JaxAnswer       jax_checklist(spread_id) -> Vec<CheckItem>
show_native_menu(items, at) -> Option<ItemId> // lifted from Waypoint (see 06)
```

Events: `export://progress`, `autosave://saved`, `fonts://changed`, `jax://ready`.

## Window chrome (D-03)

Copy Waypoint's `packages/chrome` as **`@liminal-hq/folio-chrome`** (D-08), with its tokens renamed to Folio's own (D-09):

- `WindowTitleBar`: exactly centred title; GNOME, KDE (Breeze), Cinnamon and Windows 11 control styles; `buttonLayout` read from the desktop (`org.gnome.desktop.wm.preferences button-layout` and KDE's `kwinrc`); transparency support.
- `AppMenuButton` in the title bar's start slot opens **the app menu** (see 06).
- `MenuBar` is **hidden by default**. Preferences → Interface → Show menu bar turns it on.
- `ContextMenu` and `HostedContextMenu` in-page menus have the native-menu seam (see 06).
- `WindowMenu` gives Alt+Space / right-click on title bar: Move, Size, Minimise, Maximise, Always on top, Close.

Frameless window with `decorations: false`; on Windows 11 enable snap layouts via the maximise button hit-test (`WM_NCHITTEST` → `HTMAXBUTTON`), as Waypoint does.

## Theming (D-04)

Folio's own tokens (`--folio-*`, warm stone, derived from Waypoint's native-flex set; D-09) are used throughout. The theme follows the system colour scheme via `xdg-desktop-portal` `org.freedesktop.appearance color-scheme` on Linux and `UISettings` on Windows. Users can override this with Light, Dark or Follow system. The accent comes from the system where available (portal `accent-color`, Windows accent), and falls back to Waypoint orange `#f97316` / `#c2410c`. **Book pages are never themed**: paper stays paper.

## Threading

- The UI thread never blocks: layout, export and import run on Rust worker threads (`tokio::task::spawn_blocking`).
- Layout is incremental per spread. The canvas asks for the visible spread plus its neighbours.
- Export is a job graph (layout → rasterise/convert images → write files). Targets that share work (Kindle + Kobo) share intermediate artefacts.
- Native menu popups run on a blocking worker (Waypoint pattern) so the main loop keeps painting.

## Fonts

Bundled: Literata (OFL), Atkinson Hyperlegible (OFL, for picture books and large-print), Source Serif 4 (OFL), Inter (OFL). System fonts are available, but preflight flags fonts whose licence (`OS/2.fsType`) forbids embedding.

## Testing

- `folio-model` and `folio-layout`: property tests for snapping and flow (`proptest`). Golden files for line breaking.
- `folio-export`: EPUBCheck in CI. veraPDF for PDF/X-4 and PDF/X-1a conformance.
- UI: vitest + Testing Library. The chrome is covered by its own package tests.
