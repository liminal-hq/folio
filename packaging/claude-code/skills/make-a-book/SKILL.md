---
name: make-a-book
description: Make, typeset, check and export a book with the Folio tools. Use when asked to lay out a book, a manuscript, a picture book or a KDP interior or cover.
---

# Making a book with Folio

The `folio` MCP server's tools work on books in Folio. If the person has Folio open, you are working on the book on their screen: they see each change as you make it and can undo it, so say what you are about to do.

## Steps

1. `list_kinds` shows the kinds of book and their trim sizes. Make one with `new_book` (every field is optional), or open a file with `open_book`.
2. Bring a manuscript in with `import_manuscript` (Markdown, plain text, Word, OpenDocument or Rich Text). The report lists the chapters found and what was left out. For Word or OpenDocument files, the report's styles show which Folio style each became; run the import again with `styles` to change them.
3. Read the `folio://guide/intents` resource before changing anything. `apply_intents` takes a list of intents and applies them as one undo step, or none if one is refused. `get_book` and `get_spread` give the ids the intents name.
4. Look at your work with `render_page` after each change that matters. Do not assume a layout is right; check it by eye.
5. `preflight` lists the problems a printer or store would find (pictures too small, text outside the margins, overflowing frames). Fix what it reports, or say why not.
6. `save_book` needs a `.folio` path for a new book. `export` writes the files into an `exports` folder beside the saved book.

## Things to remember

- Sizes are points (72 to the inch) from the top left of the page.
- Story text lives in flows, not in frames: change it with `SpliceStory` or `SetParagraphText`, and pages are added or removed for you as the words grow.
- `save_book` overwrites a file and `close_book` throws away unsaved changes. Ask before using either on a book the person did not ask you to make.
