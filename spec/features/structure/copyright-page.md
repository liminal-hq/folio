# Copyright page generator

**Area:** Book structure · **Prototype:** `Folio Structure.dc.html` · **Status:** partly built — Book tools → Book structure → **Copyright page** (Rust: `ToolRequest::Copyright`, `ToolFix::CopyrightPage`) makes the words from the year, holder, rights line, the book's title and paperback ISBN and the printing, and puts them in a text frame (`copyright`) on a new left-hand page after the title page, or in the frame it made before; the CIP block and a rights-holder per edition are not built

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
