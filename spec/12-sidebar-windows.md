# 12 · Sidebar, windows and layouts

## Sidebar groups (D-12)

| Group | Panels | Easy | Expert |
|---|---|---|---|
| **Properties** | Whatever is selected: page / text / picture / shape / table / multi / master / cover | ✓ | ✓ |
| **Pages** | Page list (cover first) · Page background (type, swatches, scope) | ✓ | ✓ |
| **Proofing** | Every misspelt word in the book, grouped by spread: show it, ignore it in this book, add it to the dictionary, or change it to a suggestion · the words this book ignores, each with Restore to check it again · the dictionary in use, and a better one to download | ✓ | ✓ |
| **Design** | Styles (sample, usage count, click to apply) · Swatches (+ add, CMYK/sRGB note) · Typography (hyphenation, kerning, widows/orphans, OpenType) | | ✓ |
| **Structure** | Layers · Master pages (page numbers, running heads) · Text flow (reading order, chains) · Notes, glossary and index | | ✓ |
| **Book** | Pictures in this book (needed px for 300 ppi, status) · Book details (title, subtitle, author, ISBNs, keywords, description = blurb) · Preflight, live · History (click to step back) | | ✓ |

**Built so far:** the right-hand card has two groups, **Properties** (the inspector) and **Pages**, as tabs (a `tablist`: arrow keys, Home and End move between them). Selecting something on the page brings Properties forward. Pages lists the book's spreads, cover first, each with a small swatch of what its pages show; choosing one shows it. Under the list, **Section** starts a section at the spread being shown (a name, Arabic, Roman or no numbers, and whether counting starts again, from 1 or any number; D-26) and removes it; the shelf and the `{page}` token follow the sections, so front matter reads *Pages i–ii* and the story starts again at *Page 1*, and the `{section}` token is the section's name. The cover cannot start one. Then, **Page background** lists the places a background can go on the spread being shown (both pages, each page, every page; the cover offers its own page only), with what each has now in words and as a swatch, a Change… button that opens the page background dialog (spec 11) at that place, and Remove. **Properties** for the book itself also holds a Language list, the same as the one at the top of Proofing, which picks the dictionary that checks the book. **Proofing** lists every misspelt word in the book (text frames, flowing text, the words in shapes and table cells), grouped by the spread it is on and read again a moment after the book changes. Each word can be shown (it goes to its spread and selects its frame), ignored in this book, added to the dictionary for every book, or changed to a suggestion when it sits in plain text of a frame of its own (a word in flowing text or with emphasis is changed on the page). Under the misspelt words, **Ignored in this book** lists every word the book leaves alone, each with **Restore**, which checks the word again from then on. A book whose language has no dictionary says so and offers the one to download; a book checked with a fallback offers the better one. The other groups come with the features they hold.

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

### Book tools window (D-49)

- **Built:** one `book-tools` utility window (1180 × 760, minimum 900 × 600, a child of the main window, reopening focuses it). It works on **the book the main window shows**: the main window tells the plugin which book it shows and every window hears it, so the tools window follows when another book is chosen, and a tool's change goes through the same store (one undo step of that book, shown in the main window at once).
- **Opening:** the app menu's **Book tools** submenu and the menu bar's **Tools** menu list the areas — Book structure, Typesetting checks, Pictures, Publishing helpers — and choosing one opens the window on it or moves the open window there. The ribbon's Tools tab opens it on a single tool (`open_book_tools` with a `tool`), and the window remembers the tool it was asked for until another is picked. The window's own switcher also lists Proofing and review, Series and editions, Accessibility and Picture-book extras as not built yet.
- **Shape:** a sidebar with the area switcher and that area's tools, a header with the tool's title and one line, and the tool itself; one tool at a time. With no book open it says so.

### Dialog windows (D-77)

- **What becomes a window.** A dialog with a form or a lot to read opens as a native window of its own: the Export dialog first, then the rest in turn (Book details, Barcode, Book size, Margins, Spread templates, Import report, Cover sheet, Soft proof, Check book, Page background, Story editor, Update, Connect an agent). Small bars and popovers (Find, the Marks menu, context menus, ribbon drop-downs, toasts, the Jax panel) stay in the page. A question with two or three answers (save changes before closing, restore recovered work) uses the system's message dialog (D-78).
- **Every dialog window is a modeless transient child.** A dialog window is parented to the window that opened it (a transient on Linux, owned on Windows): it stays above it and is minimised and closed with it. It is not modal. Nothing blocks, dims or covers the main window, no GTK modal and no disabled opener is used (GNOME attaches a modal dialog to its parent, so dragging the dialog dragged the main window, and a blocked window feels locked up), and the person can go on using the book beside it. Several dialogs can be open together, and opening the same dialog again focuses it rather than making a second.
- **The book keeps changing under an open dialog.** A dialog window reads its book from Rust (D-02) and follows it, so a window that shows the book (Check book, Cover sheet, Soft proof, Story editor, Page background) is current. A form that keeps a draft of its fields and commits once (Book details, Barcode, Book size, Margins) shows a notice when the book changed after the window opened, asking the person to reopen it. A dialog whose book is closed closes. Export takes a copy of the book when it starts, so editing meanwhile is safe, and it says the files show the book as it was when the export began. Spread templates adds the chosen template after the spread on show when it ends.
- **Check book** shows its list beside the book: "Show me" posts a message that moves the main window's canvas to the issue, and the list is made again as the book changes.
- **Page background previews live.** Each colour or gradient tried is applied to the book at once, so the main window beside the dialog shows it; Done keeps the tries as one undo step and Cancel or Esc rewinds the book to how it was when the window opened.
- **Update installs from the main window.** The Update window ends with "install" and the main window asks about books with changes before it restarts Folio.
- **Results.** The opener opens a dialog with a payload and waits for its result. Choosing a button ends the dialog with a result; closing the window (the ✕, Esc, or the window manager) is a cancellation, reported once. Focus goes back to the control that opened the dialog.
- **Look.** The window is frameless and shadowed exactly like the main window (D-13): the same `WindowFrame`, so on Windows it is square-edged with the OS drawing the shadow, and on Linux it has the rounded surface and CSS shadow inside the transparent margin. It has a close-only title bar, a scrolling body and a button row; Esc cancels and Enter in a field presses the default button.
- **Size and frame rules.** A dialog is never maximisable or minimisable, resizable or not, and double-clicking its title bar does nothing. Each kind has a default size, and a resizable one a minimum (a size left by the person is remembered, never below the minimum). The description, body and buttons share one bounded, centred column (720 px, `--folio-dialog-column`), so a window made larger than its content keeps the content together in the middle, with the buttons at the bottom of that column and the footer pinned to the bottom of the window.
- **Where it lands.** Windows are centred over their opener on Windows and X11; on Wayland the compositor places them. A dialog that can be resized remembers its size (`dialogs.sizes` in the settings); a position is never remembered.
- **A switch for desktops that cope badly.** Dialog windows are on by default. **Preferences → Interface → Open dialogs in their own windows** turns them off, which shows the same dialogs over the page instead, for a desktop whose windowing does not treat a dialog as a child of its opener.
- **The link dialog stays in the page.** It is a small form over the editor and its selection and focus belong to the editor's layers (still open), so it moves only once those have landed and focus has been checked.

### Tear-off panels

- Popped-out groups become **utility windows** (`always_on_top` relative to the owner, `skip_taskbar`, Waypoint chrome with only close and a Dock button). Positions are remembered per monitor.
- On Wayland, where window positions can't be set, they open as transient children and the compositor places them.
- The prototype renders them as draggable in-app windows.

## Persistence

`settings.json` (app-wide, not per book; its `export` section holds the person's default export choices, which a book's own and a single export's are laid over, D-73): `toolbarLayout`, `sidebarLayout`, `sidebarWidth`, `sidebarCollapsed`, `tornPanels: { group: { monitor, x, y, w, h } }`, `menuBar`, `quickActions`, `mode`, `theme`, `units`, `dialogs: { ownWindows, sizes: { label: { width, height } } }`.

## Accessibility

Tabs are `role="tablist"`. Rails have tooltips and accessible names. Floating windows are reachable with F6 (cycle panes) and Ctrl+F6 (cycle windows). The command search is also the keyboard-first path to every tool.

**Fonts (built):** the sidebar has a **Fonts** tab beside Properties, Pages and Proofing. It lists the typefaces the book uses, where each comes from (comes with Folio, kept in this book, on this computer), flags one that is missing (a stand-in is used) or whose licence does not let it be kept, and offers **Keep in the book** and **Stop keeping**. Choosing an installed font in the ribbon keeps it automatically when its licence allows (D-30).
