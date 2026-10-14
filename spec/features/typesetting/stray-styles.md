# Consistent styles check

**Area:** Typesetting quality · **Prototype:** `Folio Typesetting.dc.html` · **Status:** artboard → merge into Folio

## What it does
Finds local formatting that drifts from the paragraph style (e.g. one chapter in 11.5 pt), and offers to reset or turn it into a style.

## UI
List grouped by style: "Body · 4 boxes overridden (size 11.5 pt)". Buttons: Reset to style · Make new style · Update style to match.

## Model and engine
Compares frame/run overrides against `ParagraphStyle`.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
