// Identifiers for the parts of a book, as distinct string types
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

macro_rules! id {
    ($($(#[$meta:meta])* $name:ident),* $(,)?) => { $(
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
        #[ts(export, export_to = "../../../packages/protocol/src/generated/")]
        pub struct $name(pub String);
    )* }
}

id!(
    /// A book.
    BookId,
    /// A spread: one or two facing pages and the frames across them.
    SpreadId,
    /// One page of a spread.
    PageId,
    /// A frame on a spread.
    FrameId,
    /// A text flow that runs through a chain of frames.
    FlowId,
    /// A paragraph or character style.
    StyleId,
    /// A named colour a book keeps, which fields can refer to as `swatch:<id>`.
    SwatchId,
    /// A master page.
    MasterId,
);
