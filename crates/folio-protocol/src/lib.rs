// Shared types crossing the Rust <-> JS boundary. `ts-rs` generates the matching TypeScript
// interfaces into `packages/protocol/src/generated/` from this crate — never hand-maintain a
// parallel TS copy of these shapes.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod app_info;
mod window;

pub use app_info::*;
pub use window::*;
