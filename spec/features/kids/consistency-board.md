# Character and colour consistency board

**Area:** Picture-book extras · **Prototype:** `Folio Picture Book.dc.html` · **Status:** artboard → merge into Folio

## What it does
Keep characters, palettes and props consistent across illustrations.

## UI
Board with character cards (name, palette swatches, notes) and where they appear.

## Model and engine
`characters: [{ name, palette[], notes, appearsOn[] }]`.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
