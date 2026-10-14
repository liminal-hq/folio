# CMYK soft-proof

**Area:** Pictures · **Prototype:** `Folio Pictures.dc.html` · **Status:** artboard → merge into Folio

## What it does
Preview colours as they will print on the chosen paper and profile; flag colours outside the printable gamut.

## UI
View → Soft-proof toggle with profile picker; gamut warning hatch; side-by-side before/after.

## Model and engine
lcms2 transform screen→CMYK→screen per profile (GRACoL 2013, FOGRA39, KDP colour ⚠).

## Jax
Optional. Jax never interrupts this flow; he can be asked about it from the Ask tab.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
