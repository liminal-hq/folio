# Image library panel

**Area:** Pictures · **Prototype:** `Folio Pictures.dc.html` · **Status:** artboard → merge into Folio

## What it does
A tray of every picture for the book — drag onto pages, see which are used where.

## UI
Library panel with grid, filters (unused, low ppi, by chapter), drag to page or frame.

## Model and engine
Library = assets in the package plus a watched "pictures" folder.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
