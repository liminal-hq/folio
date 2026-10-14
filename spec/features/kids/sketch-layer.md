# Sketch-to-final layer

**Area:** Picture-book extras · **Prototype:** `Folio Picture Book.dc.html` · **Status:** artboard → merge into Folio

## What it does
Rough sketches on a layer under the final art, with opacity and a status per spread (sketch → inked → coloured → final).

## UI
Layer toggle, opacity slider, status chips per spread in the shelf.

## Model and engine
`frame.layer: "sketch"|"final"`, `spread.artStatus`.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
