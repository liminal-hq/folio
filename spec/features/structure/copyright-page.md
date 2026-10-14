# Copyright page generator

**Area:** Book structure · **Prototype:** `Folio Structure.dc.html` · **Status:** artboard → merge into Folio

## What it does
Fill-in-the-blanks copyright page: ©, year, author, rights statement, ISBNs per edition, edition/printing line, publisher, credits, CIP block.

## UI
Form with presets for rights wording ("All rights reserved", Creative Commons variants) and a printing line ("10 9 8 7 6 5 4 3 2 1"). Pulls ISBNs from Book details.

## Model and engine
`copyright: { year, holder, rights: "arr"|"cc-by"|…, edition, printing, publisher, credits[], cip }`; rendered into a generated page using the "copyright" style (8 pt).

## Jax
Jax explains what each line means, on request.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
