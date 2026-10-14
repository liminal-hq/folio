# KDP / IngramSpark upload checklist

**Area:** Publishing helpers · **Prototype:** `Folio Publishing.dc.html` · **Status:** artboard → merge into Folio

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
