# Translations

**Area:** Series and editions · **Prototype:** `Folio Series.dc.html` · **Status:** artboard → merge into Folio

## What it does
Keep multiple languages in one project; text frames swap language while layout stays.

## UI
Language switcher; side-by-side translate view; overflow warnings per language.

## Model and engine
`content: { "en-CA": runs, "fr-CA": runs }`; per-language hyphenation.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
