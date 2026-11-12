# Find and replace

**Area:** Proofing and review · **Status:** built (D-62) — Ctrl+F finds and Ctrl+H replaces in every story, text frame, shape, table cell and master page; Expert adds match case, whole word and regular expressions

## What it does
Finds words anywhere in the book and, when asked, replaces them. It is the same search for the person at the keyboard and for an agent (`find_text` and `replace_text`).

## UI
- **Ctrl+F** (or Home → Editing → Find & replace, or Edit → Find…) opens a bar above the page; **Ctrl+H** (Edit → Replace…) opens it with the replace row. The query is selected so typing replaces it.
- **Easy:** a find box, previous and next, the count, close; the replace row has **Replace with**, **Replace** and **Replace all**.
- **Expert:** an **Options** row opens under the box: **Match case**, **Whole word**, **Regular expression**.
- **Enter** goes to the next match, **Shift+Enter** to the previous; the page opens the spread (or the master page) the match is on and selects the words in the on-page editor. **Esc** closes the bar and gives the keyboard back to the words or the page.
- Every match on the page is marked with a soft box; the current one is stronger. Words in a locked frame are marked too, and are not changed by Replace.
- The count ("3 of 12", "No matches", "3 of 12 or more") is a polite live region. **Replace** changes the current match and moves to the next; **Replace all** is one undo step, and the bar says how many were replaced and how many were left alone (locked frames and master pages other than the one being edited).

## Rules
- Where it looks: each story, each text frame's own words, shapes, every cell of a table, groups, and master pages (their words as written, `{page}` and all).
- Order: spread, then page, then top to bottom and left to right by frame, then the place in the frame. Master pages last.
- Case folds the way Unicode does. A whole word stops at letters, digits and `_`; an apostrophe between letters is inside the word.
- Matches do not overlap; a pattern that matches no characters finds nothing.
- Regular expressions are the Rust `regex` syntax (no look-around or back-references) and always run in linear time. In **Replace with**, `$1` or `${name}` is a group of the match and `$$` a dollar sign; outside regular expression mode every character of the replacement is literal.
- The replacement takes the look (bold, italic, font, colour) of the words it replaces and is not run through smart punctuation.
- A query is at most 1 000 characters, a pattern at most 1 MiB compiled, and a search lists at most 10 000 matches.

## Model and engine
`folio_engine::find` (pure, in `find.rs`); `Books::find` and `Books::replace` (one batch of `EditText`s, one undo step); `find` and `replace` commands of `plugins/folio-books`; `find_text` and `replace_text` MCP tools. A match is `{ target, para, from, to, spread, master, context, locked }` in code points of a paragraph, so it can be sent back as an `EditText`.

## Jax
Optional. Jax can be asked to find a name that appears under two spellings.
