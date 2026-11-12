# Dialog windows: manual checklist

The mock runtime cannot parent a window, so the platform behaviour of the dialog windows (D-77) is checked by hand. Make sure **Preferences → Interface → Open dialogs in their own windows** is on (it is by default), open a book, and use **File → Export…** (or any dialog that has moved to a window) on each platform below.

For each platform check that:

- [ ] The dialog opens as its own window, above the main window, centred over it (Windows and X11) or placed by the compositor (Wayland).
- [ ] The main window stays fully usable while any dialog is open: clicks, scrolling, typing and its title bar buttons all work, and nothing dims or covers it (D-79).
- [ ] The dialog stays above the main window, and minimises and closes with it.
- [ ] Dragging a dialog's title bar moves the dialog only, never the main window.
- [ ] Opening the same dialog again (the ribbon button, the shortcut) focuses it rather than making a second.
- [ ] Double-clicking the dialog's title bar does nothing: it is not maximised, and it has no minimise or maximise button.
- [ ] Making a resizable dialog large keeps its content in a centred column with the buttons at the bottom of that column, and no inner card or second shadow is drawn inside the window.
- [ ] Editing the book beside Book details, Barcode, Book size or Margins shows "The book has changed since this window opened"; closing the book closes them; editing during an export shows the export notice.
- [ ] Esc closes the dialog, and so does its ✕ and the window manager's close; the opener hears a cancellation.
- [ ] Enter in a field presses the default button; Enter in a multi-line field adds a line.
- [ ] Tab stays inside the dialog.
- [ ] After the dialog closes, keyboard focus is back on the control that opened it, and typing goes there at once.
- [ ] Choosing "Check the book first" closes the dialog and opens Check book in the main window.
- [ ] Resizing the dialog and closing it, then opening it again, restores the size. A position is never remembered.
- [ ] A file picker or message box started from inside the dialog belongs to the dialog, not the main window.
- [ ] The window opens quickly: its content is painted within about 300 ms (the log line `first paint … ms after opening` says how long).

- [ ] Check book: "Show me" moves the main window to the issue without closing the list.
- [ ] Page background: the main window shows each colour tried; Cancel puts the page back.

## Windows 11

- [ ] The opener is focused before the dialog goes away: no other application's window comes forward when the dialog closes.
- [ ] The dialog stays above its owner and is not in the taskbar.

## Linux, Wayland: GNOME (Mutter)

- [ ] The compositor places the dialog, it stays above the main window, and it is not attached: dragging it leaves the main window where it is (D-79).

## Linux, Wayland: KDE (KWin)

- [ ] KWin keeps the dialog above the main window as an ordinary transient.

## Linux, X11

- [ ] The dialog is centred over the main window and stays above it; the window manager does not let the main window be raised over it.

## Known gaps

- A dialog's size comes from a default per kind (and the size it was left at), not from measuring its content.
- Asking for a dialog again while its window is open focuses it only where the opener asks the plugin again; a page whose own state already says "open" does not ask twice.
- macOS is not a target; a dialog opens as a child window.
