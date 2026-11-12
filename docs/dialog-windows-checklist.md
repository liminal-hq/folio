# Dialog windows: manual checklist

The mock runtime cannot parent or block a window, so the platform behaviour of the dialog windows (D-77) is checked by hand. Make sure **Preferences → Interface → Open dialogs in their own windows** is on (it is by default), open a book, and use **File → Export…** (or any dialog that has moved to a window) on each platform below.

For each platform check that:

- [ ] The dialog opens as its own window, above the main window, centred over it (Windows and X11) or placed by the compositor (Wayland).
- [ ] A modal dialog (Book size, Margins, Book details, Barcode, Page background, Import report, Update, Connect an agent): the main window cannot be used while it is open: clicks, scrolling, typing and its title bar buttons do nothing, and pressing it brings the dialog forward.
- [ ] The main window dims while a modal dialog is open (inside its visible edge, not in the shadow margin) and clears when it closes.
- [ ] Dragging a dialog's title bar moves the dialog only, never the main window.
- [ ] Opening the same dialog again (the ribbon button, the shortcut) focuses it rather than making a second.
- [ ] Esc closes the dialog, and so does its ✕ and the window manager's close; the opener hears a cancellation.
- [ ] Enter in a field presses the default button; Enter in a multi-line field adds a line.
- [ ] Tab stays inside the dialog.
- [ ] After the dialog closes, keyboard focus is back on the control that opened it, and typing goes there at once.
- [ ] Choosing "Check the book first" closes the dialog and opens Check book in the main window.
- [ ] Resizing the dialog and closing it, then opening it again, restores the size. A position is never remembered.
- [ ] A file picker or message box started from inside the dialog belongs to the dialog, not the main window.
- [ ] The window opens quickly: its content is painted within about 300 ms (the log line `first paint … ms after opening` says how long).

- [ ] Modeless dialogs (Export, Check book, Cover sheet, Soft proof, Story editor, Spread templates): the main window is not dimmed and stays usable beside them, they stay above it and minimise and close with it, and for Check book and "Show me" moves the main window to the issue without closing the list.
- [ ] Page background: the main window is not dimmed behind it and shows each colour tried; Cancel puts the page back.

## Windows 11

- [ ] The opener is enabled and focused before the dialog goes away: no other application's window comes forward when the dialog closes.
- [ ] The dialog stays above its owner and is not in the taskbar.
- [ ] The title bar buttons of the main window cannot be used while the dialog is open.

## Linux, Wayland: GNOME (Mutter)

- [ ] The compositor places the dialog, it stays above the main window, and it is not attached: dragging it leaves the main window where it is (D-79).

## Linux, Wayland: KDE (KWin)

- [ ] KWin treats the dialog as an ordinary transient, so the main window's own blocker (the dim) is what stops input: pressing the main window brings the dialog forward.

## Linux, X11

- [ ] The dialog is centred over the main window and stays above it; the window manager does not let the main window be raised over it.

## Known gaps

- macOS is not a target; a dialog opens as a child window and only the page's blocker stops the opener.
