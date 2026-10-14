# Back matter: about the author, acknowledgements, also-by

**Area:** Book structure · **Prototype:** `Folio Structure.dc.html` · **Status:** artboard → merge into Folio

## What it does
Templates for the pages after the story: acknowledgements, about the author (with photo), also by, a note to readers, reading-group questions.

## UI
Back matter step list (same wizard shell as front matter). "Also by" pulls from the Series panel.

## Model and engine
`backMatter: [{ kind, text, photo? }]` → generated pages after the last chapter.

## Jax
Jax can draft acknowledgements prompts — never writes them unasked.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
