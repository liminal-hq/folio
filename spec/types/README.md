# Document model types

The document model is implemented in [`crates/folio-model`](../../crates/folio-model) and is the source of truth. The TypeScript types the UI uses are generated from it by `ts-rs` into [`packages/protocol/src/generated`](../../packages/protocol/src/generated); never hand-edit them.

The spec's earlier sketches of the model (`project.rs` and `project.ts`) were ported into the crate and removed so there is one copy to keep right. Where a spec document names `paperback_cover()`, `Run`, `TextFrame` or the other model types, it means the ones in `folio-model`.
