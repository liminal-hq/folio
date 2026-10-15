# 12 · Sidebar, windows and layouts

## Sidebar groups (D-12)

| Group | Panels | Easy | Expert |
|---|---|---|---|
| **Properties** | Whatever is selected: page / text / picture / shape / table / multi / master / cover | ✓ | ✓ |
| **Pages** | Page list (cover first) · Page background (type, swatches, scope) | ✓ | ✓ |
| **Proofing** | Every misspelt word in the book, grouped by spread: show it, ignore it in this book, add it to the dictionary, or change it to a suggestion · the dictionary in use, and a better one to download | ✓ | ✓ |
| **Design** | Styles (sample, usage count, click to apply) · Swatches (+ add, CMYK/sRGB note) · Typography (hyphenation, kerning, widows/orphans, OpenType) | | ✓ |
| **Structure** | Layers · Master pages (page numbers, running heads) · Text flow (reading order, chains) · Notes, glossary and index | | ✓ |
| **Book** | Pictures in this book (needed px for 300 ppi, status) · Book details (title, subtitle, author, ISBNs, keywords, description = blurb) · Preflight, live · History (click to step back) | | ✓ |

**Built so far:** the right-hand card has two groups, **Properties** (the inspector) and **Pages**, as tabs (a `tablist`: arrow keys, Home and End move between them). Selecting something on the page brings Properties forward. Pages lists the book's spreads, cover first, each with a small swatch of what its pages show; choosing one shows it. Under the list, **Section** starts a section at the spread being shown (a name, Arabic, Roman or no numbers, and whether counting starts again, from 1 or any number; D-26) and removes it; the shelf and the `{page}` token follow the sections, so front matter reads *Pages i–ii* and the story starts again at *Page 1*, and the `{section}` token is the section's name. The cover cannot start one. Then, **Page background** lists the places a background can go on the spread being shown (both pages, each page, every page; the cover offers its own page only), with what each has now in words and as a swatch, a Change… button that opens the page background dialog (spec 11) at that place, and Remove. **Proofing** lists every misspelt word in the book (text frames, flowing text, the words in shapes and table cells), grouped by the spread it is on and read again a moment after the book changes. Each word can be shown (it goes to its spread and selects its frame), ignored in this book, added to the dictionary for every book, or changed to a suggestion when it sits in plain text of a frame of its own (a word in flowing text or with emphasis is changed on the page). A book whose language has no dictionary says so and offers the one to download; a book checked with a fallback offers the better one. The other groups come with the features they hold.

- **Resizable:** drag the inner edge (240–560 px). The width persists per user.
- **Collapse** (›) shrinks it to a 52 px icon rail; clicking an icon reopens that group.
- **Pop out** (↗) tears the current group off into a floating window (below).

## Sidebar layouts (View → Interface → Sidebar layout, app menu, Preferences)

| Layout | Behaviour |
|---|---|
| Right, grouped tabs (default) | Floating card on the right with tabs |
| Tool rail + inspector | Insert/format tool rail on the left, inspector on the right |
| Icon rail with flyouts | 48 px rail; clicking an icon flies its panel out over the canvas; clicking again closes it |
| Two-column dock | Properties beside the active group (Expert). Min 560 px |
| Floating panels | Every group is its own window, tiled at the edges; drag by title, Dock to return |
| Bottom drawer | Properties + active group as a 250 px drawer above the page shelf, side by side |

## Toolbar layouts (View → Interface → Toolbar layout)

Ribbon · Simplified ribbon (one row: tabs + icons) · Collapsed ribbon (tabs only; click drops it over the page; Pin) · Tool rail + top bar · Command search (type to find any tool; Enter runs the top match; no match → Ask Jax; Ctrl+K focuses it) · Menu bar + compact toolbar · Contextual only (nothing until something is selected) · Classic toolbar.

The **menu bar** stays hidden by default in every layout except "Menu bar + compact". F10 or Alt still reveals it.

## Windows

### Document tabs

- Each open book is a tab under the title bar (Waypoint tab style: colour dot by book kind, close ×, + for a new book → welcome).
- **Drag a tab down** (more than 28 px) to tear it into its own window. In the desktop app that's a new Tauri `WebviewWindow` with the same chrome, positioned under the pointer. The prototype shows an in-app mini window with **Merge back**.
- Dragging a torn window's tab bar onto another window's tab strip merges it, as in Waypoint.
- One `BookSession` per book lives in Rust. Windows are views onto sessions, so two windows can show the same book (e.g. cover and interior), with undo shared per book.

### Tear-off panels

- Popped-out groups become **utility windows** (`always_on_top` relative to the owner, `skip_taskbar`, Waypoint chrome with only close and a Dock button). Positions are remembered per monitor.
- On Wayland, where window positions can't be set, they open as transient children and the compositor places them.
- The prototype renders them as draggable in-app windows.

## Persistence

`settings.json` (app-wide, not per book): `toolbarLayout`, `sidebarLayout`, `sidebarWidth`, `sidebarCollapsed`, `tornPanels: { group: { monitor, x, y, w, h } }`, `menuBar`, `mode`, `theme`, `units`.

## Accessibility

Tabs are `role="tablist"`. Rails have tooltips and accessible names. Floating windows are reachable with F6 (cycle panes) and Ctrl+F6 (cycle windows). The command search is also the keyboard-first path to every tool.
