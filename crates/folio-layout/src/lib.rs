// The layout engine: fonts, shaping, line breaking and flow
//
// Pure Rust with no Tauri: it takes the document model and says where every line of text goes.
// Fonts come from `FontBook` (the bundled families, and the system's on request) and text is
// shaped with `shape`, with distances in points.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

pub mod assets;
pub mod breaker;
pub mod error;
pub mod families;
pub mod flow;
pub mod fonts;
pub mod hyphenate;
pub mod paragraph;
pub mod preflight;
pub mod shape;
pub mod spread;
pub mod styles;
pub mod wire;

pub use assets::{AssetFact, AssetFacts};
pub use error::LayoutError;
pub use fonts::{
    BookFace, Embedding, EmbeddingRights, FaceInfo, Font, FontBook, FontRequest, FontSource,
    KeepableFace, Metrics, Resolved,
};
pub use shape::{feature, shape, Glyph, Run, Shaped};
pub use spread::Layouter;
pub use styles::StyleSheet;
