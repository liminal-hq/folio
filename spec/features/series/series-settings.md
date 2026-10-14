# Series settings

**Area:** Series and editions · **Prototype:** `Folio Series.dc.html` · **Status:** artboard → merge into Folio

## What it does
Share styles, master pages, covers and back matter across every book in a series.

## UI
Series panel: books in the series, linked style sheet (changes push to all), cover template with per-book title/number.

## Model and engine
`.folio-series` file referencing book files + shared `styles.json`.

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
