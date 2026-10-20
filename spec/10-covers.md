# 10 · Covers

Printed covers are **one sheet**: back · spine · front, joined for printing. Folio treats the cover as its own item, **first in the page shelf**, that opens a wide cover sheet. The spine sits in the middle and is sized automatically.

## Modes (stage header switcher)

| Mode | Panels (left → right) | Edge | Notes |
|---|---|---|---|
| Paperback | back · spine · front | 0.125 in bleed | KDP paperback cover PDF |
| Hardcover | back · spine · front | 0.591 in wrap + 0.394 in hinge ⚠ | Case laminate; final numbers come from KDP's template for trim × pages |
| Dust jacket | back flap · back · spine · front · front flap | 0.125 in | Flaps 3.5 in by default (IngramSpark-style). Blurb flows onto the front flap; bio and photo sit on the back flap |
| Inside covers | inside front · inside back | none | Hardcover endpapers / inside covers. Separate frame list |
| eBook | front only, with a 1600 × 2560 px crop window (1 : 1.6) | — | Exports the eBook cover image from the front panel. Everything outside the crop is dimmed |

Geometry comes from `FOLIO_LIB.coverGeo(K, pages, paper, mode, perPage)` and mirrors `paperback_cover()` in `crates/folio-model`.

## Panel-relative frames (D-11)

Cover frames are stored **relative to their panel** (`panel: 'front' | 'back' | 'spine' | 'fflap' | 'bflap' | 'ifront' | 'iback'`, `x` from the panel's left edge). When the page count or paper changes, the spine grows and **everything on the front slides along with it**. Dragging a frame across a fold re-homes it to the panel under its centre. The spine title is a `spine`-style frame that always spans the spine's width (vertical text).

## Live spine

`spine = pages × paper factor` (white 0.002252 · cream 0.0025 · groundwood 0.00235 · premium colour 0.002347 · standard colour 0.002252 in/page ⚠). The page count is the book's page count: **Add pages** adds 2 and **Remove** subtracts 2, so the spine label, sheet size and cover thumbnail update immediately. The export dialog's page count can override it for "what if" checks. Spine text is allowed at 80 pages or more (KDP's pages disagree between 79 and 80; Folio takes the stricter) ⚠. Below that, preflight says the spine will be left blank.

## Guards

- **Barcode area:** 2 × 1.2 in, 0.25 in in from the back cover's bottom-right trim corner. It is drawn as a white box with a dashed rose border. Its edges are snap targets, and any frame overlapping it gets a rose outline plus an "Over the barcode area" tag, and a preflight warning.
- **Safe area:** 0.25 in inside each panel (0.0625 in each side of the spine), drawn with the usual purple margins.
- **Folds:** dashed lines at panel boundaries. The spine folds are orange.

## Template content

Every new book gets a cover template:
- **Front:** art (full-bleed for picture, illustrated and memoir books), title, author.
- **Spine:** title · author.
- **Back:** blurb, author photo (round), short bio.
- **Flaps:** longer blurb, bio, photo.
- **Inside front:** dedication.

The **blurb is the book description**. Editing it in Book → Book details updates the back cover, and vice versa. Title and author sync the same way.

## Backgrounds

The cover background runs edge to edge into the bleed or wrap on all sides (Pages panel → Page background, or Pages tab → Page background ▾).

## Export

- Paperback and hardcover cover PDFs are built from the sheet at full size plus edge. There are no crop marks for KDP, and an optional slug for print shops.
- eBook: JPEG at 1600 × 2560 (or 2560 × 4096 in Expert) from the crop window.
- Inside covers: a separate 2-page PDF where the printer supports it. Otherwise it is noted in the export report.

## Barcode creator (Insert → Book parts → ISBN barcode, or Cover sheet → ISBN barcode…)

- **Who makes it?** *Let KDP add it*: Folio just keeps the 2 × 1.2 in area clear and guarded. *Make my own*: generates an **EAN-13 Bookland** barcode from the ISBN.
- **ISBN entry:** accepts 13 or 10 digits with or without hyphens. 10-digit ISBNs are converted to 978-prefixed 13-digit ones. Folio checks the 978/979 prefix and the check digit, offers "Fix last digit", and explains each problem in plain words. Hyphenation for display uses the ISBN International range file (`RangeMessage.xml`, bundled and updated with releases); the prototype groups the digits approximately.
- **Price add-on (EAN-5):** No price `90000` (the self-publishing default) · US $ `5xxxx` · Canadian $ `6xxxx` · or no add-on.
- **Output:** a `barcode` frame on the back panel at the barcode area, in 100% K vector bars with OCR-B human-readable digits. It is also exported as SVG/PDF. The ISBN is saved into Book details.
- **Encoding:** `FOLIO_LIB.isbnInfo / ean13 / ean5 / barcodeGeo` (L/G/R tables, first-digit parity, 11-module quiet zone, guard bars extended). The Rust equivalent lives in `folio-export::barcode`.
- **Built so far:** `folio-export::barcode` encodes an EAN-13 (L/G/R tables, first-digit parity) and the KDP cover export draws it in the barcode box when the book's paperback ISBN is valid, at the nominal 0.33 mm module, with the ISBN above and its digits below in the body face. A missing or invalid ISBN leaves the box clear for KDP to fill. **Not built:** the creator dialog, extended guard bars, OCR-B digits, the 5-digit price add-on, and putting the bars through the K-only separation as a distinct object.
