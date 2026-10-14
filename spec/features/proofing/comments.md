# Comments and sticky notes

**Area:** Proofing and review · **Prototype:** `Folio Proofing.dc.html` · **Status:** artboard → merge into Folio

## What it does
Pin notes to places on the page, reply, and resolve them.

## UI
Proof mode toggle: sticky pins on the page, a comments column, resolve/reopen, filter by person.

## Model and engine
`comments: [{ id, anchor: {spread, x, y}|{frame, offset}, author, text, replies[], resolved }]` in the package; never printed.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
