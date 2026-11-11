# tauri-plugin-window-dialogs

Dialogs that are windows of their own. The app asks for a dialog window by label with a JSON payload; the plugin builds it as a child of the asking window, blocks the asking window while it is open where the platform allows, and sends back the dialog's result (or a cancellation, if the window was closed).

## How it works

- `open_dialog(label, spec, payload)` builds the window and returns a request id. The same label while it is open focuses it; a second modal dialog from the same window is refused.
- The dialog window asks for its own request with `dialog_request()` when it starts, so there is no race between opening and the payload arriving.
- `finish_dialog(result)` sends the result to the opener on `window-dialogs://finished` and closes the window. Closing the window any other way sends the same event with `cancelled: true`.
- `post_to_opener(message)` sends a message to the opener while the dialog stays open, for dialogs that do not block.
- `window-dialogs://modal` tells the opener that it is blocked, and whether the platform does the blocking (`native`) or the page has to.
- `Options::on_resized` lets the app remember a dialog's size.

## Modality

| Platform | How the opener is blocked                                                                                                                                                                               |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Windows  | The dialog is owned by the opener, which is disabled with `EnableWindow`, then enabled and focused before the dialog is destroyed.                                                                      |
| Linux    | The dialog is a transient of the opener and a GTK modal. GNOME attaches it; KWin and wlroots treat it as an ordinary transient (GTK 3 has no `xdg_dialog_v1`). Compositors place the dialog on Wayland. |
| Other    | Not blocked; the page's blocker is the fallback.                                                                                                                                                        |

The mock runtime cannot parent or block a window, so the tests open dialogs without an opener and the platform behaviour is checked by hand.
