# Front matter wizard

**Area:** Book structure · **Prototype:** `Folio Structure.dc.html` · **Status:** artboard → merge into Folio

## What it does
A step-by-step builder for the pages before chapter one: half-title, title page, copyright, dedication, epigraph, contents. Each step previews the real page and can be skipped.

## UI
Wizard dialog (Insert → Book parts → Front matter…). Left: step list with ticks. Centre: live page preview at trim. Right: fields. "Add these pages" inserts them in reading order before the first chapter, on rectos where convention expects (title, copyright verso, dedication recto).

## Model and engine
`frontMatter: { halfTitle: bool, title: bool, copyright: bool, dedication: string|null, epigraph: {text, source}|null, contents: bool }` on Book; generated spreads carry `generated: "front:<part>"` so they can be regenerated without losing edits flagged `edited: true`.

## Jax
Jax can walk through it ("want me to set up the first few pages?") but every step works without him.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
