# Pre-order and launch planner

**Area:** Publishing helpers · **Prototype:** `Folio Publishing.dc.html` · **Status:** artboard → merge into Folio

## What it does
A simple timeline back from the launch date: proof copy, final files, pre-order, launch day.

## UI
Timeline with dated steps and reminders (local notifications), export to calendar (.ics).

## Model and engine
`launch: { date, steps: [{ title, due, done }] }`.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
