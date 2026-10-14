# Print cost and royalty calculator

**Area:** Publishing helpers · **Prototype:** `Folio Publishing.dc.html` · **Status:** artboard → merge into Folio

## What it does
Estimates printing cost and royalty per sale for paperback/hardcover/eBook across markets.

## UI
Inputs: pages, trim, ink, list price; outputs per marketplace; break-even and minimum price.

## Model and engine
KDP formulas ⚠ (60% of list − printing cost; eBook 35%/70% bands) kept in a dated, updatable table.

## Jax
Jax explains each number on hover, on request.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
