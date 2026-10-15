# 04 · Layout engine

## Units and coordinates

- Stored in **points** (`Pt(f64)`). Shown in inches (default en-CA/en-US) or millimetres (Preferences, or the in/mm toggle in the Expert inspector).
- Spread space: origin at the top-left of the **trim** of the left-most page. The x axis runs across both pages, and the gutter sits at `x = trim.w`. The bleed area is `[-bleed, pages·w + bleed] × [-bleed, h + bleed]`.
- Single-page spreads (cover, title page) have one page. Fixed-layout spreads can be `[left, right]`, `[right]` (opener) or `[left]` (closer).

## Model

- **Book** → **Spreads** → **Pages** (each page references a **Master**) + **Frames** (spread-level, so they can cross the gutter).
- **Frame** types: `text`, `image`, `shape` (rect, round, ellipse, line, and speech, thought and shout bubbles; the "note box" is a round shape), `group`. Common fields: `box`, `rotation`, `locked`, `z` (array order), `fromMaster`.
- **Text frame**: `style`, `content` (inline runs) or `flow` membership + `flowIndex`, `columns`, `inset`, `verticalAlign`, `baselineLock`.
- **Image frame**: `asset`, `fit` (`fill` | `fit` | `stretch` | `manual`), `focus` point, `crop`, `effectivePpi` (computed).
- **Master page**: frames with `token` text (`{page}`, `{book.title}`, `{chapter.title}`, `{section}`) and ruler guides, applied per page. A master belongs to the book, not to Easy or Expert: page numbers are two managed frames on masters A and B, and any other frame or guide on a master is the person's own and is never rebuilt. Spreads may hide master items per page (`masterOverrides`), and in Expert a page can take its own editable copy of a master item (page menu, "Copy … to this page"): the item is hidden on that page and a frame the person owns sits where it was. `{page}` and `{book.title}` are filled in for every text frame, so a copied page number keeps counting. See D-22.

## Text

- Shaping with `rustybuzz`; line breaking with Knuth–Plass (greedy fallback for live typing; K–P recomputed after 150 ms idle).
- Hyphenation per language (`hyphenation` crate, en-CA/en-GB/en-US/fr-CA patterns). Minimum word length 6, at most 2 hyphens in a row, none in headings.
- Widows/orphans: default 2/2 for body; poetry uses stanza-keep instead.
- Poetry style sets `wrap: preserve`. Lines never re-wrap in print; overlong lines hang-indent and raise a preflight item.
- **Flow:** a flow is an ordered list of frames. When text overflows the last frame, the frame shows the **overset marker** (a small `+` tab at the bottom-right). In flowing books Folio can auto-add pages ("Add pages as I type", on by default in Easy).
- Drop caps, small caps and OpenType features (`liga`, `onum`, `smcp`, `ss01`…) are Expert style properties.

## Grid and guides

- **Margins** per page (top, bottom, inside, outside). Inside margins grow with page count when "Adjust gutter for page count" is on, following KDP's guidance (24–150 pages: 0.375 in; 151–300: 0.5 in; 301–500: 0.625 in; 501–700: 0.75 in; 701–828: 0.875 in — **verify against current KDP docs before release**).
- **Column grid**: `columns`, `gutter`. Drawn as an orange 7% fill with hairline edges.
- **Baseline grid**: `increment` (pt), `offset`. Drawn as cyan hairlines; text frames may lock to it.
- **Bleed line**: dashed rose at the bleed box. **Margins**: dashed purple. **Smart guides**: solid cyan 1 px.
- **Ruler guides** (Expert): drag from rulers, lockable, stored per spread or on the master.

## Built so far

`crates/folio-layout` (D-20): the font book (bundled families plus the system's, variable weights and optical sizes, `fsType` embedding check), shaping with kerning, ligatures, OpenType features and tracking, Knuth–Plass line breaking with a first-fit fallback, hyphenation (en-CA/US, en-GB, French; minimum word length 6, at most two hyphens in a row, none in headings), alignment and first-line indent, poetry that keeps its line breaks, flow through frames and columns with widows and orphans and a baseline grid, overset detection, and preflight for overset text, text outside the safe margin and pictures short of the bleed. `layout_spread` is exposed by the books plugin. Not built yet: wrap around pictures, drop caps, small-caps styles, the reflow preview, picture resolution checks and the snapping maths' use from the UI.

## Snapping (lifted from Spindle's `SceneCanvas`)

Spindle's menu editor uses an 8 px screen-space threshold and snaps the dragged node's left/centre/right (and top/middle/bottom) to the edges and centres of other nodes, with safe-area guides. Folio keeps that algorithm and adds book targets.

```
threshold = 8 px / scale                      // in document units, so it feels the same at any zoom
targets.x = per page: [left, left+margin, centre, right−margin, right]
          ∪ column edges (when grid shown)
          ∪ bleed edges (when book has bleed)
          ∪ every other frame on the spread: [x, x+w/2, x+w]
          ∪ ruler guides
targets.y = [0, margin, h/2, h−margin, h] ∪ bleed ∪ baseline lines (when baseline lock) ∪ frames: [y, y+h/2, y+h]

move:   candidates = [x, x+w/2, x+w]  → pick the single closest (candidate, target) pair ≤ threshold, per axis
resize: candidates = the moving edge(s) only
modifiers: Alt = no snapping · Shift (resize) = keep aspect · Shift (move) = constrain to axis
```

- Guides that fire are drawn for the duration of the drag. A measurement pill shows `X · Y` (move) or `W × H` (resize) in the current units.
- Keyboard nudge: arrows move 1 pt (or 1 mm when units are mm); Shift ×10. Consecutive nudges within 600 ms coalesce into one undo step.
- Equal-spacing guides (Expert): when three or more frames sit on a row, show the distance tick between them when it matches.

## Undo

- Rust keeps an intent-level undo stack (default 200 steps, coalescing for drag/nudge/typing).
- The undo stack is also persisted in the autosave journal, so a crash loses at most 30 seconds.

## Reflow preview (Kindle/Kobo)

The same flows are laid out into an e-reader viewport (Kindle Paperwhite 6.8 in: 1236 × 1648 at 300 ppi; Kobo Clara 2E: 1072 × 1448). This uses the reader CSS that the EPUB export emits, so what she sees matches what ships. Text size steps mirror the readers' own (about ±12% per step).

## Fixed layout preview

Fixed spreads render into a device frame at the device's landscape aspect. Guides are hidden, and a note explains that readers can zoom but not reflow.

## Preflight geometry (shared with Jax's page checklist)

- Text outside the safe margin (`frame` not inside `page − margin`).
- Picture touches a trim edge but stops short of the bleed (`|edge − trim| < 0.02 in` and not `≤ −bleed`).
- Effective resolution below 300 ppi (print) / 150 ppi (warn) for each placed image.
- Overset text in any flow.
- Story text below 18 pt in fixed-layout eBooks.

## Rulers and ruler guides

- **Rulers** sit along the top and left of the spread. They scroll and zoom with it, and switch on and off with View → Rulers, Layout → Rulers, Marks, or Ctrl+R. The origin is the top-left trim corner of the left page (the back panel on a cover). Negative values run into the bleed.
- **Units**, from the corner button or by right-clicking a ruler: inches · millimetres · centimetres · points · picas · pixels (96/in). Ticks adapt to the zoom: major ticks are spaced 44–160 px apart, with 6–10 minor ticks between them. Ruler units are separate from the inspector's units.
- The selected frame's extent is tinted on both rulers.
- **Guides:** drag from the top ruler for a horizontal guide, or from the left ruler for a vertical one. While dragging, a tag shows the position in ruler units. Guides snap to margins, frame edges and other guides; otherwise they snap to a fine unit step (1/16 in, 1 mm, 6 pt, 0.5 pc, 5 px). Hold Alt to place freely.
- To move a guide, drag it. Drag it back onto a ruler to delete it; the line turns red with "Release to remove".
- Right-click a guide for: Lock · Put on every page, in Expert (moves it onto the master pages the spread uses; a vertical guide goes to the master of the page it is over, and is mirrored onto the facing master) / Only on this spread · Clear guides on this spread · Clear all guides (these two clear the spread's own guides, never the every-page ones, which are removed from the master view) · Ruler units · Delete. Locked guides are dashed and don't move.
- **Storage:** `spread.guides: [{ axis: 'x'|'y', pos: points, locked }]`, on the cover's own spread too. Every-page guides (`master.guides`, in the page's own points) are kept on the master, not the spread (D-22). They are drawn on every page that uses the master in the margin colour, are snap targets, and follow the Guides mark; they cannot be dragged on a page, and are moved, locked and deleted in the master view. **Put on every page** moves a guide onto the masters (a vertical one onto the master of the page it is over, and mirrored onto the facing master); **Only on this spread**, in the master view, puts it back on the spread on show. A guide on the cover, in the gutter or off the pages cannot go on every page. Moving and deleting guides is undoable, one step each. See D-21.
- Ruler guides are snap targets for frames, and appear as their own row (with colour) in Marks.
