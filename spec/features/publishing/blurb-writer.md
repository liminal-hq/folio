# Sales-page blurb writer

**Area:** Publishing helpers · **Prototype:** `Folio Publishing.dc.html` · **Status:** artboard → merge into Folio

## What it does
Drafts a short book description from the book's own text — locally, with Jax, only when asked.

## UI
Blurb panel: tone chips (warm, playful, literary), length, "Draft with Jax", edit, compare drafts. Never auto-publishes.

## Model and engine
Tier-B local model (07) with the first chapter summary as context; output stays a draft.

## Jax
Optional — Jax drafts, you decide.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
