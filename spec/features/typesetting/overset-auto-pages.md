# Overset text and auto-add pages

**Area:** Typesetting quality · **Prototype:** `Folio Typesetting.dc.html` · **Status:** artboard → merge into Folio

## What it does
Warns when words don't fit, shows how many lines are hidden, and adds pages (threaded frames) automatically in flowing books.

## UI
Red + tab at the bottom-right of the last frame with "+12 lines"; banner "Add 2 pages"; toggle "Add pages as I type".

## Model and engine
`flow.autoAddPages`; overset lines count from layout.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
