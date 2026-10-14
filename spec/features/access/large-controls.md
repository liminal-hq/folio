# Larger-controls mode

**Area:** Accessibility · **Prototype:** `Folio Accessibility.dc.html` · **Status:** artboard → merge into Folio

## What it does
Makes the app itself bigger: 44 px targets, larger text, stronger focus rings.

## UI
Preferences → Interface → Larger controls; live sample.

## Model and engine
CSS scale tokens (`--wp-row-height` 28→44, base 13→16 px).

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
