# Widow, orphan and runt fixer

**Area:** Typesetting quality · **Prototype:** `Folio Typesetting.dc.html` · **Status:** artboard → merge into Folio

## What it does
Finds single lines stranded at the top or bottom of a page and very short last lines, highlights them, and offers one-click fixes.

## UI
Typesetting checks panel: list of issues with page, a pink highlight on the page, fixes: "Track −5 on this paragraph", "Allow one line short", "Pull a line back".

## Model and engine
`checks.widows/orphans/runts` computed by folio-layout; fix = paragraph-level override `{ tracking, keepLines, shortPage }`.

## Jax
Jax can run the checks and narrate them ("two little stragglers on 14 and 31").

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
