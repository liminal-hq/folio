# Print cost and royalty calculator

**Area:** Publishing helpers · **Prototype:** `Folio Publishing.dc.html` · **Status:** partly built — Book tools → Publishing → **Print cost and royalties** (Rust: `ToolRequest::Royalty`) works out the printing cost from the page count, then the royalty on Amazon.com (60%) and expanded distribution (40%), the lowest price that does not lose money, and the ebook royalty (70% from $2.99 to $9.99 less $0.15, else 35%). The printing costs are a dated **estimate** (`PrintCosts::estimate`, 2026-10-21): KDP's own pages disagree about the fixed cost, so Expert can enter the figures KDP's calculator gives ⚠ — check every figure against KDP's live pages before a release. Other markets and currencies are not built

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
