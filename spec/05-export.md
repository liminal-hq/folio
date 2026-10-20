# 05 · Export pipeline

One **Export book** dialog builds many targets in one pass. Each target has plain-language options (Easy) and full settings (Expert), and every target runs preflight first.

> **Verify every KDP and Kobo number below against the live help pages before release.** They change. Values here reflect our understanding as of Oct 2026 and are marked ⚠ where they need checking.

## Targets

| Target             | Output                                                         | Notes                                                                                                                                                                                                                                                                        |
| ------------------ | -------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Kindle**         | `<slug>-kindle.epub`                                           | KDP accepts EPUB uploads and converts them to Kindle formats. Reflowable for flowing books. Fixed-layout picture books are produced as fixed-layout EPUB 3, and the user is pointed to Kindle Kids' Book Creator / Kindle Create if KDP rejects direct fixed-layout upload ⚠ |
| **Kobo**           | `<slug>.epub` + `<slug>.kepub.epub`                            | EPUB 3 for Kobo Writing Life. KEPUB adds `koboSpan` wrappers for faster page turns and better stats                                                                                                                                                                          |
| **KDP paperback**  | `<slug>-paperback-interior.pdf` + `<slug>-paperback-cover.pdf` | PDF/X-1a or X-4, fonts embedded, no crop marks, bleed when any element bleeds                                                                                                                                                                                                |
| **KDP hardcover**  | `<slug>-hardcover-interior.pdf` + `<slug>-hardcover-cover.pdf` | Case laminate; wrap-around cover with hinge ⚠                                                                                                                                                                                                                                |
| **Print-shop PDF** | `<slug>-print-pdfx4.pdf`                                       | PDF/X-4 (default) or X-1a, crop marks + bleed + slug optional, chosen output intent                                                                                                                                                                                          |
| **Sharing PDF**    | `<slug>-share.pdf`                                             | RGB, images downsampled to 150 ppi, bookmarks from chapters, spreads or single pages                                                                                                                                                                                         |

Files land in `~/Books/<Title>/exports/` by default (Documents\Books on Windows). The done state lists them and offers **Show in folder**, which opens Waypoint if it's installed, otherwise the system file manager.

## EPUB 3 writer

- OCF zip (`mimetype` stored first), `content.opf` (EPUB 3.3), `nav.xhtml`, optional `toc.ncx` for older readers.
- One XHTML file per chapter (flowing), or per page (fixed layout).
- **Reflowable:** semantic HTML (`<section epub:type="chapter">`, `<h1>`, `<p>`, `<figure>`/`<figcaption>`), a small CSS with relative units, no fixed font sizes on body text, and `font-family` with publisher fonts embedded only when the licence allows. Images are max 1600 px on the long edge (Expert: 2560 or original), JPEG q85 / PNG.
- **Fixed layout:** `rendition:layout pre-paginated`, `rendition:spread landscape|auto`, per-page `viewport` meta, absolutely positioned frames, text kept as live text where possible (so it stays accessible). Spreads that cross the gutter are split into a left page and a right page, with the image sliced at the gutter.
- Accessibility metadata: `schema:accessMode`, `accessibilityFeature`, `accessibilitySummary`. Alt text comes from each image frame's description (preflight asks for missing ones, gently).
- Cover image: 1600 × 2560 px JPEG (KDP's recommended ratio 1.6:1 ⚠), generated from the front cover.
- Validated with **EPUBCheck** in CI. The export job can optionally run a bundled EPUBCheck (Java not required if we ship the GraalVM native build; otherwise skipped with a note).

## KEPUB

KEPUB is EPUB plus `<span class="koboSpan" id="kobo.N.M">` around sentences, plus a `kobo.js`-free stylesheet. The file extension is `.kepub.epub`. It's generated from the same XHTML in a post-pass.

**Built so far:** the Export tab's **Export book** opens a dialog with the Sharing PDF and the KDP paperback interior and reflowable EPUBs for Kindle and Kobo (the Kobo one with its KEPUB beside it); the print-shop PDF, the cover files and the fixed-layout EPUB are still to come; it says how many things are waiting on you with a way to check the book first, makes the chosen files in the book's `exports` folder (beside its file, or in its folder under `Books` before it is saved), lists them with their sizes and **Show in folder**. **Quick PDF** makes the Sharing PDF at once. File names follow `<slug>-share.pdf` and `<slug>-paperback-interior.pdf`. Pictures, fonts kept in the book and the page scene (D-29) are what it draws. The KDP paperback interior is the interior only (the cover is its own file), as single pages with an even page count (a blank one closes an odd count); when anything bleeds (a background, or a picture or shape running past the trim) every page is set up with the book's bleed on its outer edge, with the TrimBox and BleedBox set, and a book with no bleed is at the trim size. The **Print shop PDF (PDF/X-4)** is every page with the bleed all round, crop marks in 24 points of slug, the trim and bleed boxes, an sRGB output intent with its profile, and the PDF/X-4 version keys and XMP. Colour as ink (CMYK) is available to the exporter through `folio-colour` but not yet a choice in the dialog, The **Paperback cover for Amazon KDP** is one sheet (the bleed, the back, the spine, the front, the bleed) as wide as the pages and the paper make the spine, in CMYK: the front is the cover as it is, the back carries the book's description and a clear box for the barcode, and the spine carries the title and author, turned to read down it, from 80 pages. It is made from the cover page, so it follows what is there; The **Cover picture for a store** is the front cover drawn by Folio's own rasteriser (the same scene, the same glyph outlines) as a 1600 × 2560 JPEG, scaled to fit with the cover's colour filling the room round it, and with a thin mid-grey border when the cover is light. The **Picture book (fixed-layout EPUB)** is one XHTML page for each page with a viewport the size of the page, the pictures as images, the shapes as SVG and the words as live text at the positions layout gave, so lines break where Folio broke them; the reader scales the page. Its package says `pre-paginated`, which side each page faces, and carries the cover picture. A cover sheet of its own to edit (back and spine panels, a barcode maker, hardcover and jackets) is still to come.

## PDF writer

- `pdf-writer` + `subsetter` for font subsetting, with the full glyph set for PDF/X.
- **PDF/X-4** (default for print-shop): live transparency, ICC-based output intent (GRACoL 2013 / FOGRA39 / user ICC), RGB images converted at export with `lcms2` (perceptual) or kept with an output intent (Expert choice).
- **Print colour (built):** when a paperback file is ticked the Export dialog offers *Make the paperback interior in ink (CMYK)* and an optional path to the press's CMYK ICC profile. Ink is separated through that profile, or by a plain separation without one, and text stays black ink alone; the output is untagged DeviceCMYK as KDP wants. The paperback cover is always in ink. The print-shop PDF/X-4 stays sRGB with an sRGB output intent until PDF/X-1a lands. No press profile ships with Folio.
- **Progress and stopping (built):** while the Export dialog works it says *Making file 2 of 5: Kindle (EPUB)* and offers **Stop**, which ends the export before its next file and keeps what was made. Progress is per file, sent as `folio-books://export-progress`; a single large PDF is not interruptible part-way.
- **Soft proof (built):** Export tab → **Soft proof** asks for the press's CMYK ICC file and shows the spread being looked at twice over, as drawn and as that press would print it (the page is drawn by the exporter's rasteriser and each pixel sent through the profile with Little CMS, perceptual intent), with how much of each page is a colour the press cannot reproduce. It is a dialog and not yet a toggle on the canvas, and it assumes an sRGB screen because WebKitGTK does not colour-manage.
- **PDF/X-1a:2001**: flattened transparency and CMYK-only. Offered for printers that ask for it.
- TrimBox, BleedBox and MediaBox are set correctly. Crop marks and slug are only drawn when asked for (KDP must have **no** marks).
- Interior: single pages (not spreads) for KDP, page 1 on a recto, blank versos inserted where a chapter must open on the right.
- Validated with **veraPDF** (PDF/X profiles) in CI.

## KDP cover maths

```
spine (in) = pages × paper_factor
  paper_factor: white 0.002252 · cream 0.0025 · premium colour 0.002347 · standard colour 0.002252   ⚠
paperback full cover:
  width  = bleed + back(trim.w) + spine + front(trim.w) + bleed      bleed = 0.125 in
  height = bleed + trim.h + bleed
hardcover (case laminate):
  wrap  ≈ 0.591 in each edge (15 mm), hinge ≈ 0.394 in beside the spine   ⚠ — use KDP's template table by trim × page count
spine text allowed only when pages ≥ 79 (paperback) ⚠; keep 0.0625 in clear either side of the spine fold
barcode area: 2 × 1.2 in, bottom-right of the back cover, 0.25 in from trim edges (KDP prints the ISBN barcode there)
minimum pages: 24 (paperback) ⚠, 75 (hardcover) ⚠
```

The cover builder draws a live diagram (back · spine · front, bleed/wrap dashed, barcode box) with dimensions, and the cover artwork slots snap to those panels.

## Preflight

Issues are **fix-oriented**, sorted Waiting on you → Worth a look → Fine, shown as `IssueRow`s with a 6 px dot (never solid red blocks):

| Code              | Message                                          | Fix                                                                                                                                                                                                |
| ----------------- | ------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `IMG-PLACEHOLDER` | "4 pictures are still placeholders"              | "Drop the final artwork onto each frame. Export will leave tinted boxes otherwise."                                                                                                                |
| `IMG-LOW-PPI`     | "The picture on page 4 is 180 ppi at this size"  | "It'll print a little soft. Use a larger file, or make it smaller on the page."                                                                                                                    |
| `BLEED-SHORT`     | "A picture stops at the edge on page 7"          | "Stretch it to the pink dashed line so no white sliver shows after trimming."                                                                                                                      |
| `TEXT-SAFE`       | "Words sit outside the margin on page 12"        | "Move them inside the purple lines — they might get trimmed."                                                                                                                                      |
| `OVERSET`         | "Chapter 3 has words that don't fit"             | "Add pages, or let Folio add them for you."                                                                                                                                                        |
| `SPINE-NARROW`    | "Spine is too narrow for text (under 79 pages)"  | "We'll leave the spine blank, which KDP expects."                                                                                                                                                  |
| `FONT-EMBED`      | "Gill Sans can't be embedded in eBooks"          | "Use Literata or Inter (both included), or let readers choose their own font."                                                                                                                     |
| `FONT-MISSING`    | "Zed Sans isn't on this computer or in the book" | "Pages are set in a stand-in and will break differently. Install the font, pick another, or open the book on the computer it was made on." (engine: `PreflightKind::FontMissing`, with the family) |
| `MISSING-GLYPHS`  | "No font has 中 on page 4"                       | "These are drawn as empty boxes. Use a font that has them, or take them out." (engine: `PreflightKind::MissingGlyphs`, with the characters)                                                        |
| `FXL-SMALL-TEXT`  | "Story text is 14 pt on a fixed-layout page"     | "Make it 18 pt or larger so it reads well on a phone-sized screen."                                                                                                                                |
| `ALT-MISSING`     | "3 pictures have no description"                 | "Add a short description so screen readers can share them."                                                                                                                                        |

When nothing needs attention: **"No issues found. Book looks ready to export."**

**Built so far:** the Export tab's **Check book** button opens a dialog that lays out every spread and lists what it finds in three groups (Waiting on you, Worth a look, For your information), worst first, each with the way to fix it and **Show me**, which goes to the page and selects the frame. Empty picture frames and pictures with no description are said once with a count. It covers the checks the engine has (words that do not fit, words outside the margin, pictures short of the bleed, missing and soft pictures, a missing font, an empty picture frame, no description). The Expert panel that stays open and the checks that need the exporters (spine, page count, embedding) are still to come.

## Job model

`ExportJob { targets, shared: { layout, images }, progress }`. Kindle and Kobo share the EPUB build, and paperback and print-shop share the PDF layout. Progress events go to the dialog, which can be hidden ("Hide": keep working, done toast later). Cancelling is safe at any point because temporary files are cleaned up.
