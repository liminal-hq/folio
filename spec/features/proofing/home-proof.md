# Print a proof at home

**Area:** Proofing and review · **Prototype:** `Folio Proofing.dc.html` · **Status:** artboard → merge into Folio

## What it does
Print on a home printer as a folded booklet (saddle-stitch imposition) or 2-up spreads.

## UI
Proof print dialog: Booklet · 2-up · 1-up with crop marks; sheet preview with page numbers showing imposition order.

## Model and engine
Booklet imposition: pad to multiple of 4; sheet n front = (N−2n+2, 2n−1)…

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
