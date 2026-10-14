# Remove background / auto-trace

**Area:** Pictures · **Prototype:** `Folio Pictures.dc.html` · **Status:** artboard → merge into Folio

## What it does
Cut a subject out of its background (local model) or trace line art into vectors.

## UI
Picture tab → Remove background (preview mask, refine brush) · Trace line art (threshold slider).

## Model and engine
Small ONNX segmentation model (e.g. U²-Net-lite, ~4 MB) in folio-jax runtime; output stored as a new asset with alpha.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
