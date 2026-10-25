# Walkthrough: the cover, agents, spread templates and Book tools

This is a guide to checking the work in the stack of PRs #204 to #231 (stack #218), which sits on #203. Nothing in it is merged. Use a copy of any book you care about: several steps change the open book (each is one undo step).

## Start the app

`bun run tauri:dev` from the repository root. Open a book with **File → Open**, or make a picture book from the welcome screen. Switch to **Expert** (top right) for the numbers.

## 1. Fixes you reported (#204 to #207, #206 and #231)

- **Dialog dimming (#204).** Open Preferences or any dialog. The dimming covers the window's surface, and no square edge shows round the rounded window.
- **Zoom and move with the wheel (#205).** Over a spread, Ctrl and the wheel zoom about the pointer, Shift and the wheel move sideways.
- **Sidebar tabs and the Cover sheet dialog (#206).** In Expert the sidebar tabs wrap instead of scrolling the card sideways. Open Export → Cover sheet: the dialog opens at its full size with a box the shape of the sheet while it draws.
- **Centred text (#207).** Double-click a centred text box: the editor sits where the words are drawn instead of starting at the middle.

## 2. An agent can work on your book (#208 to #214)

`folio-mcp` is a Rust MCP server for agents. With the app open, choose **Help → Connect an agent…**, add Claude Code, then run `claude mcp get folio` in a terminal. An agent now edits the same open book; its changes join the undo history. `folio-mcp install` and `doctor` set up other programs. Opener frames are counted from their own page (#215).

## 3. The cover is the whole printed sheet (#216, #217)

Select **Cover** on the shelf. The page is back cover, spine and front side by side, with the fold and safe-area guides, the barcode box and the bleed. Rulers start at the back cover's top-left.

- Add text and a picture on the back, drag one across the spine.
- Add a spread (Pages → Add) and see the front slide along with the spine; undo is one step.
- **Export → Book details**: type a description. It fills a text frame on the back (`blurb`). Edit that frame's words, then change the description again: your words stay. **Export → Cover sheet** offers **Put the description on the back cover**.
- Double-click the barcode box (it carries a grey hint when empty) to open the barcode dialog.
- An older book (schema 2) opens with its front moved to the right panel.

## 4. Spread templates (#219 to #222)

**Insert → Spread templates** (also **Pages**, and right-click a page → **Add from template…**). A gallery of 22 templates drawn at your book's size opens on your book's kind. Choose one: it is added after the spread on show (after the first page when the cover is showing), a banner says so, and one undo removes it. Not offered while a master page is edited. Left out until the features exist: drop-cap and list-based templates.

## 5. Book tools (#223 to #231)

**Folio menu → Book tools** (or the **Tools** menu with the menu bar on) opens one window that follows the book in the main window. Change books in the main window and it follows. With no book open it says so.

**Book structure**

- _Copyright page_: fill the form, **Add the copyright page**. It lands on the left-hand page after the title page. Run again with a new printing: it updates the same page.
- _Contents_: needs a story with Heading-style chapters (bring in a Markdown file with `#` headings). Adds a contents page after the copyright page.

**Typesetting checks**

- _Smart punctuation_: shows each rule's count and a before and after example. Untick a rule, **Tidy**. One undo in the main window takes it all back. (On the Hallowe'en book it finds 74 straight apostrophes. Do not tidy the real file unless you want that.)
- _Consistent styles_: lists text boxes with their own size, alignment, weight, slant or typeface. Put one or all back.

**Pictures**

- _Pictures in this book_: each picture with a status in words (numbers in Expert) and **Replace…**.
- _Alt text_: type what a picture shows, leave the box: one undo step.

**Publishing helpers**

- _Upload checklist_: what a store will still ask for, with **Copy** buttons.
- _Keywords_: seven slots with counters.
- _Print cost and royalties_: set prices. The printing costs are an **estimate** dated 2026-10-21: KDP's own pages disagree about them. Expert can enter the figures from KDP's calculator.

An agent can read the same reports through the MCP tool `book_report`.

## Not built, or not checked

- Book tools: Front matter checklist, back matter, index and glossary builders, widows and orphans, rivers and loose lines, hyphenation, linked and embedded pictures, the Soft proof entry from the Pictures area, launch planner, blurb writer, and the Proofing, Series, Accessibility and Picture-book areas. The switcher lists those as not built.
- Spread templates: Save spread as template…, the cookbook group, templates for single pages.
- Every KDP figure marked with a warning sign in the specs must be checked against KDP's live pages before a release.
- Windows: the named-pipe agent link and the new windows were not run on Windows 11.
- I checked in the running app: the cover on the stage, the templates gallery and inserting and undoing one, and the Book tools window on Publishing and Typesetting. The Structure and Pictures tools, the cover's editing, and the agent dialog were covered by tests only.
