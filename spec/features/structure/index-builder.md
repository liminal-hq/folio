# Index page builder

**Area:** Book structure · **Prototype:** `Folio Structure.dc.html` · **Status:** artboard → merge into Folio

## What it does
Turns marked index entries into a sorted, two-column index with page numbers, sub-entries and see-also references.

## UI
Index panel: list of entries (term, sub-term, pages), merge duplicates, see-also; preview of the generated index pages.

## Model and engine
`indexEntries: [{ term, sub?, anchor: {frame, offset}, seeAlso? }]`; pages resolved at layout; ranges collapse ("12–14").

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
