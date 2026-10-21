# Keywords and categories helper

**Area:** Publishing helpers · **Prototype:** `Folio Publishing.dc.html` · **Status:** partly built — Book tools → Publishing → **Keywords** has seven slots with character counters (50 each), kept as `metadata.keywords` by `SetKeywords` (one undo step, merging while typed); the category picker (BISAC) is not built

## What it does
Up to 7 keyword phrases and category suggestions, with counters and gentle guidance.

## UI
Seven keyword slots with character counters, tips, category picker (BISAC subject tree, bundled).

## Model and engine
`metadata.keywords[7]`, `metadata.bisac[]`.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
