# 11 · Page tools

New tools are spread across the existing ribbon tabs. **Contextual tabs** (Picture, Shape, Table) appear, underlined in the accent colour, when one of those is selected. They aren't switched to automatically except when the thing was just inserted.

## Page backgrounds

A background is one of four kinds, stored as `{ "type": …, … }`:

- `colour`: `{ colour: "#rrggbb" }`.
- `gradient`: `{ angle, stops: [{ at, colour }] }`, with `angle` in degrees clockwise from up, as in CSS (90 runs left to right, 180 top to bottom). A preset is stored as the stops it expands to.
- `texture`: `{ id, tint? }`, one of the bundled textures.
- `picture`: `{ asset, fit: "fill" | "fit", focus: [x, y] }`.

| Scope | Stored on | Notes |
|---|---|---|
| This page, Left or Right | `page.bg` | Reaches into the bleed on the outer edge |
| Across both pages | `spread.bgSpan` | One background through the gutter, with a picture split at the gutter for EPUB fixed layout. Not offered on a single page. |
| Every page | `master.bg` on each master in use | Pages with their own background keep it |
| Cover | `page.bg` of the cover | Whole sheet, into the bleed or wrap |

A page shows its own background, else the one across its spread, else its master's, else the paper. See D-23.

**Setting one:** the ribbon's Pages → Page background button, or "Page background…" on the page's context menu, opens a dialog. It asks where the background goes (both pages, the left or right page, or every page; a single page offers this page and every page), then what it is (Colour with the swatches and a hex box, a Gradient from the presets, a Texture with an optional tint, or a Picture). A choice applies at once, so the page behind shows it, and the tries on one page are one undo step. "Remove background" clears the chosen place. A Picture tab chooses an image (brought into the book like any picture) and whether it fills the page, cropped at the edges, or fits whole inside it. The picture's `focus` is stored but not yet used: a picture is cropped from its centre until the focus control is built.

Bundled textures are Folio's own, drawn by the app as tiles of a fixed size in points so they look the same on every book and run on through the gutter: Linen (`linen`), Laid paper (`laid-paper`), Kraft (`kraft`), Watercolour wash (`watercolour`) and Dot grid (`dot-grid`). A texture may be tinted, which replaces its base colour. A book that names a texture an older build cannot draw shows the paper, and an intent naming an unknown id is refused. Gradients: Dawn, Meadow, Dusk, Lagoon, Paper fade. A picture background is an image asset with Fill / Fit. Checks for a missing picture or too few pixels come with picture preflight, for frames and backgrounds together.

## Shapes and bubbles (Insert → Shapes / Bubbles)

A shape is `{ shape: 'rect' | 'round' | 'ellipse' | 'line' | 'speech' | 'thought' | 'shout', fill?, stroke?, strokeWidth?, radius?, flip? }` on a frame. `fill` and `stroke` are `#rrggbb`, or none. `radius` rounds a `round` shape's corners and `flip` turns a line to run from the bottom-left to the top-right.

- Paths come from `shapePath(kind, w, h)` and stretch with the frame. The stroke is non-scaling.
- Bubbles are shapes with text (`bubble` style, centred) and padding tuned per kind so the words sit in the body, not the tail.
- **Looking at a shape:** selecting one shows its look in the inspector (Easy and Expert): the kind (rectangle, rounded, ellipse, line or a bubble), the fill and the outline from the swatches or any `#rrggbb` (a fill can be none; a line has only an outline), the outline width in points, how round a rounded shape's corners are, and "Run the other way" for a line. Trying colours or widths on one shape is one undo step.
- Contextual **Shape** tab: Fill ▾, Outline ▾, Change shape ▾, rotate, wrap.
- Note box: the existing `callout` text style.

## Text wrap

Pictures and shapes have `wrap: undefined | 'around' | 'topbottom'`.
- **Around:** text boxes that overlap get a float on the nearer side, sized to the overlap plus 0.1 in. It is placed with a zero-width spacer float, so the text genuinely reflows.
- **Skip (top and bottom):** a full-width float, so text jumps past.
- In the desktop engine, wrap becomes an exclusion polygon in `folio-layout` line breaking (04). Circle corners use an ellipse exclusion.

## Picture tools (contextual Picture tab + sidebar)

| Tool | Model |
|---|---|
| Replace | new asset, keeps frame size, crop, border |
| Fit / fill | `fit: 'fit'` or default fill |
| Crop | `crop: { z: 1–3, x: −50–50 %, y: −50–50 % }`. Never destructive |
| Rotate left / right | `rot` in degrees (any frame type; free entry in Expert) |
| Flip across / down | `fx`, `fy` (image content only) |
| Border | none · hairline · thick · white mount (polaroid) |
| Corners | square · soft · round · circle |
| Filters | Original · Warm · Cool · Black & white · Sepia · Faded (applied on export via `image` crate ops; CSS filters in preview) |
| Stretch to bleed | fills the nearest page(s) to the bleed line |

## Tables (Insert → Table)

`TableFrame { rows: string[][], head: boolean, ts: 'lined' | 'grid' | 'recipe' }`

- When a table is selected, its cells are editable in place. Typing in a cell is one undo step per focus.
- Contextual **Table** tab: add/remove row and column, header row, table style.
- EPUB: semantic `<table>` with `<thead>`. PDF: laid out by `folio-layout` with row heights from content (v1.1); the prototype uses equal rows.

## Ornaments and dividers (Insert → Decorate)

Fleuron ❦, Three stars ⁂, Three dots, Wave, Simple rule. These are text frames in the `ornament` style, so they follow the book's fonts and export as text.

## Footnotes, glossary, index

See 09. They are collected in Structure → Notes, glossary and index.

## Arrange: multi-select, align, distribute, group, lock, layers

- **Shift-click** adds or removes items. **Ctrl+A** selects everything on the spread. The status bar shows "3 selected".
- **Align** (Layout tab, sidebar when multi): left edges, centres, right edges, tops, middles, bottoms. With one item it aligns to the page margins.
- **Distribute** across / down: three or more items, equal gaps inside the selection's bounds.
- **Group** (Ctrl+G) sets a shared `g`. Clicking any member selects the group, and they move together. **Ungroup** is Ctrl+Shift+G.
- **Lock** (`lock`) still selects but won't drag or nudge.
- **Layers panel** (Structure): top-to-bottom list with hide (`hide`), lock, and forward/backward. Clicking selects; Shift-click adds.

## Spread templates (Insert or Pages → Spread templates)

The gallery dialog shows live thumbnails sized to *this* book. Choosing one inserts a new spread after the current one (+2 pages, so the spine grows) and says so in a toast.

| Group | Templates |
|---|---|
| Picture book | Full-bleed spread · Picture + words · Vignettes |
| Cookbook | Recipe page (photo, label, title, meta, bulleted ingredients, numbered method) |
| Textbook | Lesson + worksheet (objectives callout, body, worksheet table, figure + caption) |
| Photos | Photo collage (5 photos + caption) |
| Poetry | Poem, centred (ornament on the verso) |
| Chapter openers | Classic (drop cap) · Photo · Minimal (big numeral) |

Templates are data (`FOLIO_LIB.TEMPLATES[].build(K)`), so user templates can be saved the same way: **Save spread as template…** stores the frames normalised to trim in `templates/*.json` inside the `.folio` package, or app-wide in `~/.local/share/folio/templates`.
