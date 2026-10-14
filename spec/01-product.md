# 01 · Product

## Who it's for

**Primary:** a capable non-specialist making her own books, such as children's picture books, a family memoir, a recipe collection or a small poetry book. She uses Windows 11, writes first in Word or Google Docs, and wants physical copies from Amazon KDP plus eBooks on Kindle and Kobo. She should never need to know what "PDF/X-4" means, but she should be able to find out.

**Secondary:** experienced self-publishers and small presses who outgrew Word and find Scribus or InDesign heavy. They live in Expert mode.

## Book kinds (presets)

Each kind is a preset bundle: trim sizes, margins, bleed, grid, styles, master pages, a sample layout and export defaults.

| Kind | Layout | Default trim | Bleed | Grid | Notes |
|---|---|---|---|---|---|
| Picture book | Fixed | 8.5 × 8.5 in | 0.125 in | 6 col | Pictures lead, big story text (≥ 18 pt), no folios by default, spreads that run across the gutter |
| Novel / all-text | Flowing | 5 × 8 in | none | 1 col | Running heads, folios, chapters open on the recto, drop caps optional, widow/orphan control |
| Textbook / course | Fixed | 8.5 × 11 in | 0.125 in | 12 col + baseline | Figures, captions, callouts, numbered steps, glossary sidebars, figure numbering |
| Illustrated (cookbook, garden, craft) | Flowing with anchored images | 7 × 10 in | 0.125 in | 6 col | Recipe/project blocks: title, meta line, ingredients/materials, method |
| Poetry | Flowing, line-preserving | 5.5 × 8.5 in | none | 1 col | Generous margins, never re-wraps a line in print, stanza-aware page breaks |
| Memoir with photos | Flowing + photo spreads | 6 × 9 in | 0.125 in | 6 col | Photo spreads with captions, running heads by chapter |

Each kind ships **sample layouts** (3–6 spreads) that the user can start from or copy spreads out of.

## Modes (D-01)

The welcome screen asks **"How do you like to work?"**: Easy or Expert. The choice can be changed at any time from the toolbar, the app menu or Preferences, and Folio remembers it per user, not per book.

| | Easy | Expert |
|---|---|---|
| Inspector | Words, style chips, Smaller/Bigger, alignment, picture fit, Stretch to bleed | Adds X/Y/W/H, units, margins/bleed/columns/gutter, leading, tracking, OpenType, threads, master page editing |
| Ribbon | Core groups only | Adds Masters, Type grid, Import layout, Justify |
| Export | Targets + plain-language options + preflight | Adds PDF standard, output intent, image downsampling, font embedding policy, crop marks/slug |
| Language | "Pink dashed line (bleed)" | "Bleed 0.125 in" |

Easy never removes data. Expert-only settings that are already in a book keep working and show a small "Set in Expert mode" hint.

## Core features (v1)

1. **Welcome:** pick a book kind, size and mode, decide whether Jax helps, open recents, or import.
2. **Spread editor:** frames (text, picture, note box), drag with smart snapping, four-corner resize, keyboard nudge, undo/redo, in-place text editing, right-click menus.
3. **Page shelf:** spread thumbnails with drag to reorder, plus add, duplicate and remove.
4. **Styles:** paragraph and character styles; each preset seeds a small, named set.
5. **Master pages:** folios and running heads, with per-spread overrides.
6. **Text flow:** threaded text frames; reflow on edit; overset indicator.
7. **Preview:** Print, Kindle and Kobo. Reflowable books show a live reader view with adjustable text size. Fixed layouts show the spread in a device frame.
8. **Export:** Kindle (EPUB for KDP), Kobo (EPUB 3 + KEPUB), KDP paperback (interior + cover PDF), KDP hardcover, print-shop PDF/X, sharing PDF. Multiple targets build in one pass.
9. **Cover builder:** spine width from page count and paper, barcode-safe area, bleed/wrap, front/back/spine artwork slots.
10. **Preflight:** fix-oriented checks before export (see 05).
11. **Import:** Word, Google Docs, ODT, RTF/Pages, Markdown, paste with formatting; Scribus, IDML and EPUB in Expert (see 08).
12. **Jax:** optional companion with gentle tips, per-page checklist, guided walk-through, and offline "how do I…?" answers (see 07).
13. **Projects** are `.folio` files: open, documented zips (see 03).

## Non-goals (v1)

- Real-time collaboration and any required cloud service.
- Vector illustration tools beyond simple shapes. Use your drawing app and drop the file in.
- macOS builds.
- Direct upload to KDP or Kobo (no public APIs). Folio produces files and a checklist.

## Principles applied

- **Local first:** nothing leaves the machine unless exported. Jax's model runs locally.
- **Sovereignty:** `.folio` is a plain zip of JSON + assets. Unpacking it must always yield usable files.
- **Calm:** no nagging. Preflight shows "worth a look" items, never alarm red. There is always a "That's everything." closure.
- **Explain the why:** every warning says what happened and how to fix it.
