// The Folio document model: the book, its spreads and frames, and the maths and intents around them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//
// The model is the source of truth and lives in Rust (D-02). Every type that crosses to the UI
// generates TypeScript into `packages/protocol`. All lengths are points (1/72 in).

mod background;
mod book;
mod colour;
mod cover;
mod flow;
mod frame;
mod ids;
mod intent;
mod sections;
mod snap;
mod spread;
mod units;

pub use background::*;
pub use book::*;
pub use colour::*;
pub use cover::*;
pub use flow::*;
pub use frame::*;
pub use ids::*;
pub use intent::*;
pub use sections::*;
pub use snap::*;
pub use spread::*;
pub use units::*;
