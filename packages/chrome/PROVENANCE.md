# Provenance

This package is a copy of `packages/chrome` from [liminal-hq/waypoint](https://github.com/liminal-hq/waypoint), published there as `@liminal-hq/waypoint-chrome`, taken at commit `a3c9894728da69a9b44e66310e66cde64c10ea60`.

## Changes from the source

- The package is `@liminal-hq/folio-chrome`.
- Waypoint's `--wp-*` custom properties are Folio's `--folio-*`, and `data-wp-*` attributes are `data-folio-*` (decision D-09). The rename is the only change to the sources; it is a plain find and replace over `src/`, so later fixes can be brought across the same way.
- The `AppMenuButton` test uses the label "Folio".

## Why a copy

The chrome is not published. When it is (or when a shared Liminal workspace owns it), delete this copy and depend on the release instead (decision D-08).
