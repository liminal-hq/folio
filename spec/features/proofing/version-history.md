# Version history and compare

**Area:** Proofing and review · **Prototype:** `Folio Proofing.dc.html` · **Status:** artboard → merge into Folio

## What it does
Named snapshots and a compare view that shows what changed between two versions.

## UI
History panel with snapshots ("Sent to Mum, 4 Oct"), compare: side-by-side spreads with change highlights and a text diff.

## Model and engine
Snapshots stored as compressed book.json diffs in `/versions/` inside the package.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
