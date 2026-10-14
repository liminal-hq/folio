# Real image import and resolution check

**Area:** Pictures · **Prototype:** `Folio Pictures.dc.html` · **Status:** artboard → merge into Folio

## What it does
Drop real pictures onto frames; Folio shows effective ppi at the placed size and warns before print.

## UI
Drop zone on frames and the library; ppi badge (green ≥ 300, amber 200–299, red < 200) live while resizing.

## Model and engine
`asset { path, sha256, px: {w,h}, colour: "rgb"|"cmyk"|"grey", icc? }`; effectivePpi = px / (frame inches × crop zoom).

## Jax
Jax can explain "why does 180 ppi look soft?".

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
