# 09 · Text tools

## Where they live

| Surface | Contents | Shown |
|---|---|---|
| **Home tab** | History · **Font** (family, style, size −/+, B I U S, small caps, x² x₂, colour ▾, highlight ▾, clear) · **Paragraph** (align ×4, bullets, numbering, outdent/indent, first-line indent, line spacing ▾, space after ▾, drop cap, letter spacing ▾ *Expert*) · **Editing** (Find & replace, Spelling) · Help (Ask Jax) | Always (Text tab in the Contextual layout) |
| **Mini toolbar** | Style picker · B I U · A− A+ · colour · link · Ask Jax | Floats above a selected or edited text box; flips below when there's no room |
| **Sidebar · Properties** | Words (Easy), style chips, font, Smaller/Bigger, character buttons, colour swatches, alignment, list, line spacing, space after, first-line indent, drop cap, indent | When a text box or bubble is selected |
| **Classic toolbar** | Font family + size box inline with B I ⟸ ≡ ⟹ | Classic / Menu bar layouts |

Toolbar buttons use `onMouseDown: preventDefault` while editing, so the text selection survives a click.

## Granularity (D-10)

- **Easy:** formatting applies to the **whole box**, even while typing. The first time someone tries to style a selection, a one-off toast explains that Expert mode can style single words.
- **Expert:** while typing, formatting applies to the **selection**. With no selection, or when the box is just selected, it applies to the whole box.
- Superscript, subscript and links need a selection, so they always need Expert mode plus a text selection.

## Model: runs

```ts
interface Run { t: string; b?: boolean; i?: boolean; u?: boolean; st?: boolean; sc?: boolean; sup?: boolean; sub?: boolean; c?: string; hl?: string; href?: string }
interface TextFrame { text: string; runs?: Run[]; /* box-level */ s: StyleId; font?: string; pt?: number; b?; i?; u?; st?; sc?; c?; hl?; ls?: number /*em*/;
  al?: Align; lh?: number; sa?: Pt; ind?: Inches; fi?: Inches; list?: 'bullet' | 'number'; dc?: boolean; ot?: string[] }
```

- `text` is always the plain text, kept for search, the reader preview and EPUB fallbacks. `runs` exists only when the box has mixed styles.
- Box-level properties are defaults; run properties override them. **Clear formatting** removes both.
- Paragraphs are `\n`-separated. List, indent, first-line indent, space after and drop cap are paragraph-level and apply to every paragraph in the box (v1). Per-paragraph overrides are planned for 1.1 and will be stored as `paras: [{ list, ind, … }]`.
- Rust: `Run` / `TextFrame` in `folio-model` (see `crates/folio-model`); `runs` serialise as `content.inline` (03).

## Editing

- Double-click (or Enter) opens a `contentEditable` editor seeded from `runsToHtml(runs)`. Box-level styles apply to the editor itself.
- Commit (blur, Esc, clicking the page) parses the DOM back with `domToRuns` (tags B/I/U/S/SUP/SUB/A plus inline `font-weight`, `font-style`, `text-decoration`, `font-variant`, `color` and `background-color`). Adjacent equal runs are merged, and if nothing is rich the result collapses back to plain `text`.
- In the desktop app the editor is a Rust-shaped text view rather than `contentEditable` (02). The run model is the same.
- Ctrl+B/I/U work while typing and on a selected box.
- **Undo while typing.** Undo takes back one stretch of typing at a time, not the whole editing session. The words typed so far are kept as an undo step after a pause of a second in the typing, before and after a paste or a cut, before and after a correction from the spelling menu, and before anything else changes the book (a style, a move, an undo or redo). The editor stays open through all of these. Undo and redo from the ribbon, menu or keyboard shortcut close the editor first, so that what is on the page and what the book holds are the same. Ctrl+Z with the caret in the box is the box's own: it takes back the last typing in the box without touching the book.

## Paragraph details

| Control | Values |
|---|---|
| Line spacing | 1.0 · 1.15 · 1.3 · 1.5 · 1.8 · 2.0 (multiplier of size) |
| Space after | 0 · 4 · 8 · 12 · 18 pt |
| Indent / outdent | ±0.25 in left indent |
| First-line indent | toggles 0.25 in (the body style already indents after the first paragraph) |
| Lists | Bullet `•` or `1.` numbering with a 1.3 em hanging marker. Numbering counts non-empty paragraphs |
| Drop cap | First letter floats at 2.15 × line height, weight 600, warm ink |
| Letter spacing (Expert) | Tight −0.02 · Normal · Loose 0.03 · Very loose 0.08 em |
| OpenType (Design panel) | liga · onum · smcp · swsh · frac → `font-feature-settings` |

## Fonts

The picker offers the bundled families (Literata, Source Serif 4, Atkinson Hyperlegible, Inter) plus "From style" and system fonts. Preflight warns when a font forbids embedding (05).

## Find & replace and spelling

- **Find & replace** (Ctrl+F or Home → Find & replace): Find, Replace with, a live count ("3 found in this book"), Next (jumps to the spread and selects the box), Replace, and Replace all (one undo step, with a toast saying how many matches it changed).
- **Spelling** (D-25): wavy underlines under misspelt words, while typing and on the page (text frames, flowing text, the words in shapes and the cells of tables), from Folio's own checker rather than the webview's or the system's, so the language is the same on Linux and Windows and Rust-owned text can be checked too. It is on by default, and the Home tab's Spelling button and Preferences turn it off. The language is the book's (en-CA unless the book says otherwise), and a run can carry its own `lang`. Right-click a misspelt word for suggestions, Ignore in this book (kept with the book) and Add to dictionary (kept for every book on this computer, in the settings document as `spelling.personal`, and removed again from Preferences → Spelling); Alt+F7 jumps to the next misspelling and Shift+F10 opens the same menu from the keyboard. Words ignored in a book travel with the file (`proofing.ignore`); words added to the dictionary stay with the person's settings. The English (Canada) dictionary is bundled. Preferences → Spelling lists the others (English for the United States, the United Kingdom and Australia, French, Spanish, Brazilian Portuguese and Dutch) with their size and licence, and downloads one when asked: each file is fetched over HTTPS from a pinned address, checked against its pinned size and SHA-256, read once, and only then put in place with its licence text beside it; a failed or altered download leaves nothing behind. The book's language is set from a Language list in the Properties card and at the top of the Proofing group, which offers every dictionary Folio knows (saying which are not downloaded yet); changing it is one undo step, and the spelling is checked again against the dictionary for the new language. A book is checked with the dictionary made for its language (`fr-CA` takes French), else the one for its main language, else the bundled English for any English; a language with none is not checked. Dictionaries under a GPL-only licence (German, Italian, Swedish) are not offered. Underlines are never the only sign: a live region says how many misspellings a frame has.

## Footnotes, glossary, index

- **Footnote** (Insert → Book parts): inserts a superscript number at the caret, or at the end of the box, and a `footnote`-style frame at the foot of that page. Numbering is per book. EPUB uses `epub:type="footnote"` asides.
- **Glossary term / Index entry** (Expert): select a word while typing. Terms are collected in Structure → Notes, glossary and index. The back-matter pages are generated at export.
