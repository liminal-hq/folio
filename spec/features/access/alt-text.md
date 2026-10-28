# Alt-text helper

**Area:** Accessibility · **Prototype:** `Folio Accessibility.dc.html` · **Status:** partly built — Book tools → Pictures → **Alt text** lists every picture with a box for what it shows, kept as one undo step (`SetFrameDescription`, up to 500 characters, an empty box takes it away); the context from nearby text, the examples and the decorative toggle are not built, and the Accessibility area itself is not

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
