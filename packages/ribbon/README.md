# @liminal-hq/folio-ribbon

A data-driven ribbon toolbar for Liminal HQ desktop apps. It knows nothing about books: an app hands it a `RibbonModel` (tabs → groups → items, see `spec/components/ribbon.md`) and a mode, and gets `onCommand({ id, value, item })` back for every press, toggle, menu pick, select and stepper change, plus `tab` and `mode` (and `file` for the File button's menu).

- **Variants:** `full`, `simplified` (one row) and `collapsed` (tabs only, click to drop the panel over the page, Pin to keep it open), in `comfortable` or `compact` density, with or without labels.
- **Easy and Expert:** items and groups marked `expert` are hidden in Easy mode (`visibleTabs`). Contextual tabs show only while the app lists them in `contextual`.
- **State:** toggles and values are kept by the ribbon unless the app sets `active` or `value` on the item, which then owns it. Toggles sharing a `radio` key are mutually exclusive.
- **Menus:** dropdowns and the File menu use the chrome's `ContextMenu`, so the ribbon depends on `@liminal-hq/folio-chrome` (and never the reverse).
- **Keyboard:** ←/→/Home/End move between tabs, ↓ enters the panel, ←/→ move across its controls, Esc returns to the tab (and closes a dropped-down collapsed ribbon). Buttons keep focus where it was on pointer-down, so a text selection survives a click. Alt key tips are not built yet.
- **Tooltips:** name, shortcut chip and a one-line description after 450 ms, 60 ms when moving between controls.
- **Strings:** the ribbon's own labels come through the `labels` prop (`RibbonLabels`); item labels come from the model.
