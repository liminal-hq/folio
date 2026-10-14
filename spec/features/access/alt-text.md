# Alt-text helper

**Area:** Accessibility · **Prototype:** `Folio Accessibility.dc.html` · **Status:** artboard → merge into Folio

## What it does
Walks through every picture asking for a short description, with examples and a length guide.

## UI
Alt-text queue: picture, context (nearby text), field, tips; "decorative" toggle.

## Model and engine
`frame.description`, `frame.decorative`; required by preflight for EPUB.

## Jax
Jax can suggest a first draft from the caption — optional.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
