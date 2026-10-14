# Audiobook script export

**Area:** Series and editions · **Prototype:** `Folio Series.dc.html` · **Status:** artboard → merge into Folio

## What it does
Export a narrator-friendly script: chapters, pronunciations, notes, no layout clutter.

## UI
Export target with options: include pronunciation guide, mark scene breaks, per-chapter files.

## Model and engine
Plain text / DOCX / ACX-friendly naming.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
