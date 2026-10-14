# Rivers and loose lines

**Area:** Typesetting quality · **Prototype:** `Folio Typesetting.dc.html` · **Status:** artboard → merge into Folio

## What it does
Highlights justified lines with stretched spacing and vertical "rivers" of white space.

## UI
Overlay mode: loose lines tinted by severity; rivers traced. Fixes: allow hyphenation here, small tracking, rewrite hint.

## Model and engine
Uses Knuth–Plass badness from folio-layout; threshold settings in Expert.

## Jax
Jax can point at the worst three.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
