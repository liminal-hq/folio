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
- Rust: `Run` / `TextFrame` in `folio-model` (see `types/project.rs`); `runs` serialise as `content.inline` (03).

## Editing

- Double-click (or Enter) opens a `contentEditable` editor seeded from `runsToHtml(runs)`. Box-level styles apply to the editor itself.
- Commit (blur, Esc, clicking the page) parses the DOM back with `domToRuns` (tags B/I/U/S/SUP/SUB/A plus inline `font-weight`, `font-style`, `text-decoration`, `font-variant`, `color` and `background-color`). Adjacent equal runs are merged, and if nothing is rich the result collapses back to plain `text`.
- In the desktop app the editor is a Rust-shaped text view rather than `contentEditable` (02). The run model is the same.
- Ctrl+B/I/U work while typing and on a selected box.

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

- **Find bar** (Ctrl+F or Home → Find & replace) sits under the toolbar: Find, Replace with, live count ("3 found in this book"), Next (jumps to the spread and selects the box), Replace, Replace all (one undo step, with a toast saying how many matches it changed).
- **Spelling:** wavy underlines while typing, using the system checker (Hunspell on Linux, the Windows spell checker on Windows, via `tauri-plugin-spellcheck`), with suggestions on right-click. It is off by default so the page stays calm. The toggle lives in the find bar and the Home tab.

## Footnotes, glossary, index

- **Footnote** (Insert → Book parts): inserts a superscript number at the caret, or at the end of the box, and a `footnote`-style frame at the foot of that page. Numbering is per book. EPUB uses `epub:type="footnote"` asides.
- **Glossary term / Index entry** (Expert): select a word while typing. Terms are collected in Structure → Notes, glossary and index. The back-matter pages are generated at export.
