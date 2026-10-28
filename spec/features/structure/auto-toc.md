# Auto table of contents

**Area:** Book structure · **Prototype:** `Folio Structure.dc.html` · **Status:** partly built — Book tools → Book structure → **Contents** (Rust: `ToolRequest::Contents`, `ToolFix::ContentsPage`) lists the chapters of the story (paragraphs in the Heading style) with the label of the page each begins on, and sets them as a heading and two aligned columns (`toc_heading`, `toc_titles`, `toc_pages`) on a new page after the copyright page; run it again to bring the numbers up to date. Depth, dotted leaders, number position and chapter order are not built

## What it does
Contents built from chapter headings, with real page numbers that update as pages move.

## UI
Contents page frame (style "toc") with dot leaders; options: depth (chapters / + sections), leader style (dots, none), numbers right/after. Refresh is automatic on layout; a "Contents is up to date" badge.

## Model and engine
TOC entries are derived, never stored as text: `tocFrame { depth, leaders, align }`; layout resolves `{heading → page}` after pagination. EPUB uses nav.xhtml (no page numbers).

## Jax
Optional tip when a chapter has no heading style.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
