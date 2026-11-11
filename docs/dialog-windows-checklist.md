# Dialog windows: manual checklist

The mock runtime cannot parent or block a window, so the platform behaviour of the dialog windows (D-77) is checked by hand. Turn on **Preferences → Experimental → Open dialogs in their own windows**, open a book, and use **File → Export…** (or any dialog that has moved to a window) on each platform below.

For each platform check that:

- [ ] The dialog opens as its own window, above the main window, centred over it (Windows and X11) or placed by the compositor (Wayland).
- [ ] The main window cannot be used while the dialog is open: clicks, scrolling, typing and its title bar buttons do nothing, and pressing it brings the dialog forward.
- [ ] The main window dims while the dialog is open and clears when it closes.
- [ ] Opening the same dialog again (the ribbon button, the shortcut) focuses it rather than making a second.
- [ ] Esc closes the dialog, and so does its ✕ and the window manager's close; the opener hears a cancellation.
- [ ] Enter in a field presses the default button; Enter in a multi-line field adds a line.
- [ ] Tab stays inside the dialog.
- [ ] After the dialog closes, keyboard focus is back on the control that opened it, and typing goes there at once.
- [ ] Choosing "Check the book first" closes the dialog and opens Check book in the main window.
- [ ] Resizing the dialog and closing it, then opening it again, restores the size. A position is never remembered.
- [ ] A file picker or message box started from inside the dialog belongs to the dialog, not the main window.
- [ ] The window opens quickly: its content is painted within about 300 ms (the log line `first paint … ms after opening` says how long).

- [ ] Check book (the one dialog that does not block): the main window stays usable beside it, and "Show me" moves the main window to the issue without closing the list.
- [ ] Page background: the main window is not dimmed behind it and shows each colour tried; Cancel puts the page back.

## Windows 11

- [ ] The opener is enabled and focused before the dialog goes away: no other application's window comes forward when the dialog closes.
- [ ] The dialog stays above its owner and is not in the taskbar.
- [ ] The title bar buttons of the main window cannot be used while the dialog is open.

## Linux, Wayland: GNOME (Mutter)

- [ ] The dialog is attached to the main window (it moves with it) and the compositor places it.

## Linux, Wayland: KDE (KWin)

- [ ] KWin treats the dialog as an ordinary transient, so the main window's own blocker (the dim) is what stops input: pressing the main window brings the dialog forward.

## Linux, X11

- [ ] The dialog is centred over the main window and stays above it; the window manager does not let the main window be raised over it.

## Known gaps

- macOS is not a target; a dialog opens as a child window and only the page's blocker stops the opener.
