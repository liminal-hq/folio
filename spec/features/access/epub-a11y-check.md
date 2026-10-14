# EPUB accessibility checker

**Area:** Accessibility · **Prototype:** `Folio Accessibility.dc.html` · **Status:** artboard → merge into Folio

## What it does
Checks reading order, headings, contrast, language, alt text and metadata before export.

## UI
Checker list with severity dots and fixes; reading-order view numbers frames.

## Model and engine
Ace-like rules (DAISY) implemented in folio-export; WCAG contrast on text over pictures.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
