# Share a review PDF

**Area:** Proofing and review · **Prototype:** `Folio Proofing.dc.html` · **Status:** artboard → merge into Folio

## What it does
Export a light PDF with comment support for someone else to mark up, then import their notes back.

## UI
Export → Review PDF; Import comments… reads PDF annotations and places them as Folio comments.

## Model and engine
PDF with page labels + hidden anchors; annotation import maps rect → spread position.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
