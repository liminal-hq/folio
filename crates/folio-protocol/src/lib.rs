// Shared types crossing the Rust <-> JS boundary. `ts-rs` generates the matching TypeScript
// interfaces into `packages/protocol/src/generated/` from this crate — never hand-maintain a
// parallel TS copy of these shapes.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod agent;
mod app_info;
mod status;
mod update;
mod window;

pub use agent::*;
pub use app_info::*;
pub use status::*;
pub use update::*;
pub use window::*;
