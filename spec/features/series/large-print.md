# Large-print edition

**Area:** Series and editions · **Prototype:** `Folio Series.dc.html` · **Status:** artboard → merge into Folio

## What it does
Generate a large-print edition (≥ 16 pt, wider spacing) from the same book.

## UI
Edition wizard: base size, font (Atkinson), spacing; re-pagination preview; own ISBN.

## Model and engine
Edition = style overrides + trim; content shared.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
