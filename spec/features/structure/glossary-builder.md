# Glossary page builder

**Area:** Book structure · **Prototype:** `Folio Structure.dc.html` · **Status:** artboard → merge into Folio

## What it does
Collects glossary terms and their definitions into alphabetised pages.

## UI
Glossary panel: term + definition editor, alphabet jump, preview.

## Model and engine
`glossary: [{ term, definition, firstUse: anchor }]`; optional bold-on-first-use in body text.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
