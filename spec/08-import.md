# 08 · Import

Two doors:

- **Import text** (Easy + Expert) brings **words with structure** into the current book's flows and styles.
- **Import layout** (Expert) brings a **whole designed document** in as spreads and frames.

Every import ends with an **Import report**: what came across, what was approximated, and what was skipped, with the original kept in `assets/imports/` so nothing is lost.

## Import text

| Source | How | Notes |
|---|---|---|
| Word `.docx` | zip → `word/document.xml`, `styles.xml`, `numbering.xml`, `media/` | Heading 1/2/3 → chapter/section; Normal → Body; Quote → Epigraph/Block quote; lists; footnotes → endnotes (or footnotes in Expert); images → anchored picture frames; comments dropped (listed in report); tracked changes: accept all (ask first) |
| Google Docs | user exports `.docx` (preferred) or "Web page (.html, zipped)" | Same as docx/HTML. A "How to export from Google Docs" help card is linked from the dialog |
| LibreOffice `.odt` | zip → `content.xml`, `styles.xml` | ODF paragraph styles map by name and outline level |
| Pages / RTF | `.rtf` parser; Pages users export RTF or DOCX | Pages bundles aren't read directly (proprietary). The dialog says so kindly |
| Markdown / plain text | `pulldown-cmark` with Folio attributes (`{style=…}`) | `#` → chapter; blank-line paragraphs; `***` → scene break |
| Paste with formatting | clipboard `text/html` → sanitise → same mapping as HTML | Plain paste with Ctrl+Shift+V |

**Built so far:** `folio-import` reads Markdown and plain text (`pulldown-cmark`, with smart punctuation, tables and footnotes recognised so they can be reported) into the paragraphs of a story: `#` becomes a `h1` paragraph, other headings `h2`, blank lines separate paragraphs, `***` becomes a centred `* * *` in the `ornament` style, `*` and `**` become italic and bold runs, quotations take `epigraph` and list items become paragraphs that start with a dash. Pictures, tables, code, HTML and footnotes are left out, links keep their words, and the `ImportReport` says each once. **File → Import text…** (and the Insert tab's button) asks for a `.docx`, `.odt`, `.rtf`, `.md`, `.markdown` or `.txt` file (up to 8 MB), replaces the story of the book's first flow with it in one undo step, and shows the report; a book whose pages hold their own words has no story to bring them into and says so. The original file is not yet kept in `assets/imports/`. **Word (`.docx`)** is read from `word/document.xml` (`zip` and `quick-xml`): Heading 1 and Title become `h1`, Heading 2–4 and Subtitle `h2`, Quote styles `epigraph`, bold and italic runs keep their emphasis, numbered and bulleted items start with a dash, tracked insertions are accepted and deletions dropped; tables, pictures, footnotes and comments are left out and the report says so. The picker offers `.docx` too. **OpenDocument (`.odt`)** is read from `content.xml` the same way: `text:h` by outline level (1 is `h1`, others `h2`), quotation paragraph styles as `epigraph`, bold and italic from the automatic text styles of spans, list items with a dash; tables, pictures, notes, comments and tracked changes are left out and reported. **Style mapping:** a Word or OpenDocument import lists the paragraph styles the document used (name, how many paragraphs, a sample of the first) in the report with the Folio style each was given (Body text, Chapter title, Subheading, Quotation); changing a choice and pressing **Bring it in again** reads the same file again with the choices (`import_text` takes an optional map from the document's style name to a Folio style id, the empty string being body text), replacing the story again in one more undo step. Styles are named as the document names them (Word's style id such as `Heading1`; OpenDocument's `Heading 2` for headings and the paragraph style's name otherwise). It is not asked before the first import, and the choices are not remembered between files.

**Rich Text (`.rtf`)** is read by a small reader of its control words: paragraphs, bold and italic, escapes, dashes and quotes; font, colour and style tables, pictures and notes are skipped. RTF does not tell headings or quotations apart, so every paragraph comes in as body text and the report says to use Word or OpenDocument to keep them. The `{style=…}` attributes and HTML paste are not built.

### Style mapping dialog (Easy shows a summary; Expert shows the table)

"We found 4 kinds of text in your document" → each source style is shown next to the Folio style it will use (Chapter title → Heading, Normal → Body, Caption → Caption, Intense Quote → Note box), with a live sample. Easy just says "Looks good?" with Change… for adjustments.

### Chapter detection

Heading 1, or the page-break + bold-centred-line heuristic for documents without styles. A chapter can start on a recto (novel default), on the next page, or continue.

## Import layout (Expert)

| Source | Status | What maps |
|---|---|---|
| **Scribus `.sla`** | v1 | XML: `DOCUMENT` (page size, bleed, margins), `MASTERPAGE`, `PAGEOBJECT` (text frames with `StoryText`, image frames, shapes), paragraph/character styles, layers (flattened, listed), linked text frames (`NEXTITEM`) → flows. Colours → swatches (CMYK preserved) |
| **IDML** (InDesign, Affinity Publisher export) | v1 | zip of XML: `designmap.xml`, `Spreads/*.xml`, `Stories/*.xml`, `Resources/Styles.xml`, `MasterSpreads`. Text frames, threads, rectangles/images, paragraph/character styles, swatches. Unsupported (reported): tables (v1.1), anchored objects inside tables, GREP styles, scripts |
| **EPUB** | v1 | Reflowable → flows + styles from CSS (best effort) → re-layout into the chosen kind. Fixed-layout → one spread per page pair with absolutely positioned frames |
| Affinity `.afpub` | not planned | Proprietary; use IDML export from Affinity Publisher 2 |
| PDF | not planned (v1) | The report suggests importing the original text instead |

### Fidelity rules

- Geometry is converted to points exactly; rotated frames keep their rotation.
- Fonts that are missing on this machine get a substitute (shown with a dotted underline in Expert) and are listed in the report with "Find font…".
- Anything not understood is **kept** in `book.json` under `extensions.<source>` so that a later Folio version can do better, and so that nothing silently disappears.

## Import report (example)

```
Imported "Watercolour Basics.idml"
✓ 24 spreads, 2 master spreads
✓ 31 text frames in 6 threads · 214 paragraphs
✓ 18 pictures (4 linked files found next to the IDML, 14 embedded)
• 3 tables imported as pictures — editable tables arrive in Folio 1.1
• Font "Freight Text Pro" isn't installed — using Literata for now
That's everything.
```
