# Smart quotes, dashes and ellipses

**Area:** Typesetting quality · **Prototype:** `Folio Typesetting.dc.html` · **Status:** built — as you type and paste (`editing.smartPunctuation`, on by default, D-57), on import and in Book tools; Book tools → Typesetting checks → **Smart punctuation** (Rust: `ToolRequest::Punctuation`, `ToolFix::Punctuation`) finds straight quotes and apostrophes, `--` and spaced hyphens, three dots and double spaces in every story and text box, shows how many places and an example of each as it is and as it would be, and tidies the ticked rules in one undo step (the words keep their look: `SetParagraphText` and `SetText`); table cells are covered and the quote style follows the book's language

## What it does
Cleans typewriter punctuation: straight to curly quotes, double hyphens to em dashes, number ranges to en dashes, three dots to ellipses, double spaces to single.

## UI
Clean-up dialog with before/after diff per change type, counts and "Apply all" (one undo step).

## As you type (D-57)
On by default, a personal setting: **Preferences → Editing → Smart punctuation as I type**, the **Smart punctuation** toggle in the Home tab's Editing group (pressed while it is on) and **Edit → Smart punctuation as I type** (also in the app menu) all set `editing.smartPunctuation`. In the on-page editor typed and pasted plain words are set by the rules below as they go in, in one `edit_text` call; the older text box is unchanged. **Backspace straight after a conversion puts back what was typed** (`--` after the dash, the straight quote after a curly one) as one more undo step, and the next characters are left as typed. Words pasted with Folio's own formatting, Enter, deletions and input-method compositions are not changed.

## Rules (`folio_model::smarten`)
- Quotes open at a paragraph start or after a space, opening bracket, dash or opening quote, and close otherwise; an apostrophe closes inside a word and before digits (’90s).
- `--` is an em dash (more hyphens join it), a hyphen between spaces an en dash, `...` one ellipsis. Hyphens in words and a minus at the start are left alone.
- Characters already set the way a book sets them are left alone.

## Language
The quote style comes from the book's language (`book.language`, or a run's own `lang` when it has one, D-65: a quote typed after words in French is French in an English book, and the Book tool sets a paragraph by the language all of its words share): English and en-CA “ ” ‘ ’; French and fr-CA « » with non-breaking spaces inside and ‘ ’ within; German „ “ ‚ ’; any other language gets the English style.

## Import
Word, OpenDocument and Rich Text imports and Markdown are set by the same rules when the setting is on, so `--` is an em dash everywhere; code spans are left as written.

## Model and engine
One rule set in `folio-model` (`smart.rs`), used by `edit_text` (`EditRequest.smart`, answered with a `SmartNote`), the importers and the Book tool (`folio-engine/src/punctuation.rs`, which also tidies double spaces); applied to runs without losing formatting.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
