# Alt-text helper

**Area:** Accessibility · **Prototype:** `Folio Accessibility.dc.html` · **Status:** partly built — Book tools → Pictures → **Alt text** lists every picture with a box for what it shows, kept as one undo step (`SetFrameDescription`, up to 500 characters, an empty box takes it away); the **decorative** tick is built (below); the context from nearby text and the examples are not, and the Accessibility area itself is not

## What it does
Walks through every picture asking for a short description, with examples and a length guide.

## UI
Alt-text queue: picture, context (nearby text), field, tips; "decorative" toggle (built: a tick under each picture's box).

## Model and engine
`frame.description`, `frame.decorative`; required by preflight for EPUB.

## Decoration (built)

A frame the person marks `decorative` is decoration: the Alt text tool has a tick for each picture, and the Properties panel has **Mark as decorative** for pictures and for the words of a text frame, one undo step (`SetFrameDecorative`, also available to the engine and the MCP server). A tagged PDF marks decoration as an artifact so a screen reader skips it, and preflight does not ask for a description of it. An empty description never means decorative: that is what the "no description" note is for.

Folio also treats two things as decoration without being told, because a picture book's page numbers are usually plain text frames laid over small pictures. A **page number** is a text frame whose whole text is a number (digits, roman numerals or the `{page}` token, with or without dashes or brackets round it), no wider than 40% of the page and no taller than 12%, whose middle is in the bottom 15% of its page, at a place (same side of the spread, within 2 points) where another page also has such a number, so a lone number is never guessed at. A **page-number backdrop** is a picture with no description, no longer than 20% of the page's shorter side, that lies behind a page number and holds its middle. A picture the person described is content whatever it sits under. The Alt text tool says when it is treating a picture this way, and ticking **Decorative** there or in the panel makes it explicit. The rule is `folio_layout::decoration`, shared by preflight and the PDF exporter.

## Jax
Jax can suggest a first draft from the caption — optional.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
