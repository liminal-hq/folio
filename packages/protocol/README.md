# @liminal-hq/folio-protocol

The TypeScript side of Folio's Rust wire types. Every file in `src/generated/` is written by `ts-rs` from a `#[ts(export)]` type in `crates/folio-protocol`, `crates/folio-model` or another domain crate; nothing here is hand-written and nothing here has behaviour, so the package is types only.

## Using it

Import a type from its deep path, with no barrel file:

```ts
import type { WindowKind } from '@liminal-hq/folio-protocol/generated/WindowKind';
```

## Adding or changing a type

Annotate the Rust type with `#[ts(export, export_to = "../../../packages/protocol/src/generated/")]` (the path is relative to the crate's `src/`), then run `bun run bindings` and commit the regenerated files. CI runs `bun run check:bindings`, which deletes this directory, regenerates it and fails on any difference.

Reusable plugins that graduate to the shared plugins workspace export into their own `guest-js/bindings` instead and never import from this package.
