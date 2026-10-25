# KDP / IngramSpark upload checklist

**Area:** Publishing helpers · **Prototype:** `Folio Publishing.dc.html` · **Status:** partly built — Book tools → Publishing → **Upload checklist** (Rust: `ToolRequest::Checklist`) lists title, author, description, language, keywords, ISBN, page count (KDP's 24 to 828), cover, problems with the pages and missing pictures, each done, to do or worth a look, with how to put it right and a **Copy** button for the words; IngramSpark and Kobo field maps and the file paths of the exports are not built ⚠ (verify the fields against the stores' current forms)

## What it does
A checklist that mirrors the upload forms, prefilled from the book, with copy buttons for every field.

## UI
Checklist with sections (Details, Content, Rights & pricing), tick as you go, copy-to-clipboard per field, file paths for the exports.

## Model and engine
Generated from Book metadata + exports; per-platform field maps (⚠ verify fields against current forms).

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
