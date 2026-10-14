# Linked vs embedded assets

**Area:** Pictures · **Prototype:** `Folio Pictures.dc.html` · **Status:** artboard → merge into Folio

## What it does
Keep originals inside the .folio (embedded) or link to files on disk; relink missing files in one go.

## UI
Assets list with status chips (Embedded · Linked · Missing · Modified); "Find missing in folder…", "Embed all".

## Model and engine
`asset.storage: "embedded"|"linked"`, `linkPath`; watcher marks Modified when the file changes.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
