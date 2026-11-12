# tauri-plugin-window-dialogs

Dialogs that are windows of their own. The app asks for a dialog window by label with a JSON payload; the plugin builds it as a modeless transient child of the asking window (it stays above it, and is minimised and closed with it, but blocks nothing) and sends back the dialog's result (or a cancellation, if the window was closed).

## How it works

- `open_dialog(label, spec, payload)` builds the window and returns a request id. The same label while it is open focuses it rather than making a second; different dialogs can be open together.
- The dialog window asks for its own request with `dialog_request()` when it starts, so there is no race between opening and the payload arriving.
- `finish_dialog(result)` sends the result to the opener on `window-dialogs://finished` and closes the window. Closing the window any other way sends the same event with `cancelled: true`.
- `post_to_opener(message)` sends a message to the opener while the dialog stays open.
- `Options::on_resized` lets the app remember a dialog's size.
- A dialog is never maximisable or minimisable.

## Parenting, not modality

| Platform | How the dialog belongs to its opener                                                                                                                                                                                                                      |
| -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Windows  | The dialog is owned by the opener (`parent`); the opener stays enabled, and is focused before the dialog is destroyed.                                                                                                                                    |
| Linux    | The dialog is a transient of the opener (`gtk_window_set_transient_for`, `xdg_toplevel.set_parent` on Wayland). It is not a GTK modal: GNOME attaches a modal dialog to its parent and moves it with the parent. Compositors place the dialog on Wayland. |
| Other    | A child window, placed by the app.                                                                                                                                                                                                                        |

The mock runtime cannot parent a window, so the tests open dialogs without an opener and the platform behaviour is checked by hand.
