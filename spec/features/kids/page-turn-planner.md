# Page-turn reveal planner

**Area:** Picture-book extras · **Prototype:** `Folio Picture Book.dc.html` · **Status:** artboard → merge into Folio

## What it does
See the whole book as page turns and plan what each turn reveals.

## UI
Storyboard of all spreads with a "reveal" note per turn and a tension line.

## Model and engine
`spread.reveal: string`, `spread.beat: 0–5`.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
