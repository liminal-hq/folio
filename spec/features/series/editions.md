# Different trim per edition

**Area:** Series and editions · **Prototype:** `Folio Series.dc.html` · **Status:** artboard → merge into Folio

## What it does
Paperback, hardcover and eBook editions with their own trim, margins and ISBN.

## UI
Editions bar in the title area; switch to see each edition laid out.

## Model and engine
`editions: [{ id, kind, trim, margins, isbn, overrides }]`.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
