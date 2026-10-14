// What can go wrong when loading fonts and shaping text
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("no font is available for {family:?}, and no fallback could be loaded")]
    NoFont { family: String },
    #[error("the font {family:?} could not be read")]
    BadFont { family: String },
}
