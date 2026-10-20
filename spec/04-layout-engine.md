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
- **Master page**: frames with `token` text (`{page}`, `{book.title}`, `{chapter.title}`, `{section}`) and ruler guides, applied per page. A master belongs to the book, not to Easy or Expert: page numbers are two managed frames on masters A and B, and any other frame or guide on a master is the person's own and is never rebuilt. Spreads may hide master items per page (`masterOverrides`), and in Expert a page can take its own editable copy of a master item (page menu, "Copy … to this page"): the item is hidden on that page and a frame the person owns sits where it was. `{page}` (the number as the page's section writes it: `7`, `vii`), `{section}` and `{book.title}` are filled in for every text frame, so a copied page number keeps counting. See D-22 and, for sections, D-26.

## Text

- Shaping with `rustybuzz`; line breaking with Knuth–Plass (greedy fallback for live typing; K–P recomputed after 150 ms idle).
- Hyphenation per language (`hyphenation` crate, en-CA/en-GB/en-US/fr-CA patterns). Minimum word length 6, at most 2 hyphens in a row, none in headings.
- Widows/orphans: default 2/2 for body; poetry uses stanza-keep instead.
- Poetry style sets `wrap: preserve`. Lines never re-wrap in print; overlong lines hang-indent and raise a preflight item.
- **Flow:** a flow is an ordered list of frames. When text overflows the last frame, the frame shows the **overset marker** (a small `+` tab at the bottom-right). In flowing books Folio can auto-add pages ("Add pages as I type", on by default in Easy).
- **A flow's story (D-39):** `Flow.paragraphs` is a list of `StoryPara { style?, inline: Run[] }`, the same runs a text frame holds, so emphasis, font and colour carry. A paragraph's `style` is given to its runs that name none, and a paragraph is set as its first words' style says, so a heading in the middle of a story has the room a heading has. Paragraphs are named by their place in the list. Books written with a `source` string read as one paragraph for each line, and a `source` that is a path to a package text file carries no words. A story is edited with four intents, each one undo step and typing in one paragraph merging into one: `SetParagraphText` (keeps the look of the words left alone), `SpliceStory` (Enter, Backspace across paragraphs, paste and deleting paragraphs), `StyleParagraphs` and `StyleStoryRange`. They reach the copy in the UI as `SetFlow` / `RemoveFlow`, which carry the whole flow, so a very long story is sent whole on each change (a per-paragraph patch is the way to speed that up). Frames are chained with `LinkFrame { frame, to }` (the frame follows `to`; `to` starts a flow, with its words as the story, when it was not in one, and the words `frame` held go to the end of the story) and taken out with `UnlinkFrame` (a frame that was the only one takes the story back as its own words); deleting a frame takes it out of its flow, and each is one undo step. In Expert the Inspector's **Continue in another frame** asks for a click on an empty text frame (Escape or Cancel gives up), and **Take out of the story** unchains it. The Inspector's **Edit the story…** opens the **story editor**: a box for each paragraph with a paragraph-style list beside it. Typing sends `SetParagraphText`, Enter splits the paragraph at the caret (`SplitParagraph`) and Backspace at the start of one joins it to the one before (`JoinParagraphs`), both keeping the look of the words; paragraphs off screen are not laid out (`content-visibility`). The sample novel now pours its three body frames into one flow (a sample frame that names a `flow` joins that story, and a blank line in the sample words is the end of a paragraph), and the EPUB tells a flow's story once, where its first frame is met, with each paragraph in the style it names. **Pages follow the story:** after a change to a story (its words, their styles, the frames chained to it, or an import) the books plugin lays the flow out with `Layouter::frames_needed` (it tries more and more frames like the last one, then narrows down) and, when the story has outgrown its frames, sends `AddFlowPages { flow, count }`: that many frames like the last one, each after the one before, on the other page of the same spread when it is free and on new two-page spreads otherwise. The pages join the change in one undo step (`Session::join_last`). When the words of a story are edited and it no longer reaches its last frames, `TrimFlowPages { flow, keep }` takes the frames after the first `keep` (the ones the story reaches, at least one) out of the flow and off their spreads, and removes a spread they leave with nothing on it (no frame, background or guide; never the cover or the book's last spread), in the same undo step. Chaining a frame by hand or restyling never gives pages back. **Chapters open on a right-hand page (D-41):** a paragraph style's `start` is `page` or `recto`; a paragraph in such a style begins at the top of the next page, or the next right-hand page, and the frames between are left empty (the EPUB sets `page-break-before: always`). The Heading style says `recto`, so a novel's chapters do; a frame's own words ignore it. A story with no right-hand frame left is overset from that paragraph. **From the page:** double-clicking a frame that is in a story (not locked, not from a master) edits **the paragraph under the click** in place: a box over the frame holds that paragraph's whole words (a frame shows only a part of a story, so a paragraph that runs on from another frame is edited whole), set in the frame's font and spell-checked as other text is; each pause, paste and correction is an undo step (`SetParagraphText`, which keeps the look of the words left alone) and Escape or leaving the box ends the editing. Each line of a story's layout names its paragraph (`LayoutLine.paragraph`) so the click finds it. A new line typed in the box becomes a space; splitting and joining paragraphs, and styling them, are done in the story editor (the Inspector's **Edit the story…**), which opens instead when the click did not land on a paragraph. **Not built yet:** typing across paragraphs or frames in one go on the page (one paragraph is edited at a time), spelling in the story editor, a list of chapters to jump between, and adding pages as the story grows.
- Drop caps, small caps and OpenType features (`liga`, `onum`, `smcp`, `ss01`…) are Expert style properties.

## Grid and guides

- **Margins** per page (top, bottom, inside, outside). Inside margins grow with page count when "Adjust gutter for page count" is on, following KDP's guidance (24–150 pages: 0.375 in; 151–300: 0.5 in; 301–500: 0.625 in; 501–700: 0.75 in; 701–828: 0.875 in — **verify against current KDP docs before release**).
- **Column grid**: `columns`, `gutter`. Drawn as an orange 7% fill with hairline edges.
- **Baseline grid**: `increment` (pt), `offset`. Drawn as cyan hairlines; text frames may lock to it.
- **Bleed line**: dashed rose at the bleed box. **Margins**: dashed purple. **Smart guides**: solid cyan 1 px.
- **Ruler guides** (Expert): drag from rulers, lockable, stored per spread or on the master.

## Built so far

`crates/folio-layout` (D-20): the font book (bundled families plus the system's, variable weights and optical sizes, `fsType` embedding check), shaping with kerning, ligatures, OpenType features and tracking, Knuth–Plass line breaking with a first-fit fallback, hyphenation (en-CA/US, en-GB, French; minimum word length 6, at most two hyphens in a row, none in headings), alignment and first-line indent, poetry that keeps its line breaks, flow through frames and columns with widows and orphans and a baseline grid, overset detection, and preflight for overset text, text outside the safe margin, pictures short of the bleed, and pictures that are missing or too coarse for print (the books plugin reads each picture's size from its file header once and hands layout `AssetFacts`). `layout_spread` is exposed by the books plugin. Not built yet: wrap around pictures, drop caps, small-caps styles, the reflow preview and the snapping maths' use from the UI.

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
- A crash loses at most the autosave interval (30 seconds by default): a recovery copy of the book is kept beside the real file (D-27). A recovered book starts with no undo history.

## Reflow preview (Kindle/Kobo)

The same flows are laid out into an e-reader viewport (Kindle Paperwhite 6.8 in: 1236 × 1648 at 300 ppi; Kobo Clara 2E: 1072 × 1448). This uses the reader CSS that the EPUB export emits, so what she sees matches what ships. Text size steps mirror the readers' own (about ±12% per step).

**Built so far:** the View tab's **Kindle** and **Kobo** buttons switch the stage to a device frame of that screen (shown whole, scaled to fit). A flowing book is laid out in columns one screen wide from the very chapters and CSS the EPUB ships (`reader_preview`), with page turns, text size in steps of about 12%, white, sepia and dark pages, and a switch to the reader's own font; fonts are lent from the page and real devices differ a little, which the preview says. **Print** goes back to the pages.

## Fixed layout preview

Fixed spreads render into a device frame at the device's landscape aspect. Guides are hidden, and a note explains that readers can zoom but not reflow. This is built: the current spread is drawn read-only in the frame.

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
