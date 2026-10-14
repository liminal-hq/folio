# Hyphenation dictionary per book

**Area:** Typesetting quality · **Prototype:** `Folio Typesetting.dc.html` · **Status:** artboard → merge into Folio

## What it does
A per-book list of words with custom break points or "never hyphenate" (names, brand words).

## UI
Dictionary table: word, break points (pre·ten·tious), never-break toggle, used on pages. Import/export as plain text.

## Model and engine
`hyphenation: { lang, exceptions: { word: "pre-ten-tious" | null } }` stored in the .folio package.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
