# Endnotes vs footnotes switch

**Area:** Book structure · **Prototype:** `Folio Structure.dc.html` · **Status:** artboard → merge into Folio

## What it does
One switch moves every note between the foot of each page and a Notes section (per chapter or at the end of the book).

## UI
Segmented control: Footnotes · Endnotes by chapter · Endnotes at end. Live preview of where notes land.

## Model and engine
`notes: { placement: "foot"|"chapter"|"end", numbering: "book"|"chapter", marker: "1"|"*" }`; note bodies stored once, placed by layout.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
