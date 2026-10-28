# Smart quotes, dashes and ellipses

**Area:** Typesetting quality · **Prototype:** `Folio Typesetting.dc.html` · **Status:** partly built — Book tools → Typesetting checks → **Smart punctuation** (Rust: `ToolRequest::Punctuation`, `ToolFix::Punctuation`) finds straight quotes and apostrophes, `--` and spaced hyphens, three dots and double spaces in every story and text box, shows how many places and an example of each as it is and as it would be, and tidies the ticked rules in one undo step (the words keep their look: `SetParagraphText` and `SetText`); fix-as-I-type and table cells are not built

## What it does
Cleans typewriter punctuation: straight to curly quotes, double hyphens to em dashes, number ranges to en dashes, three dots to ellipses, double spaces to single.

## UI
Clean-up dialog with before/after diff per change type, counts, "Apply all" (one undo step) and an "as I type" toggle.

## Model and engine
Rules per language (en-CA/en-GB/fr-CA guillemets); applied to runs without losing formatting.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
