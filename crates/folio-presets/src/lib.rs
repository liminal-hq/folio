// What a new book starts from: the six kinds, their sizes and styles, and the sample pages
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Each kind of book is a preset bundle (spec 01): trim sizes, margins, bleed, grid, a named set of
// styles and a few sample spreads to start from. The bundles are data (`data/kinds.json` and
// `data/styles.json`), written once from the design prototype and owned here since; the UI asks this
// crate for them and never keeps a copy (D-18). `new_book` carries every field the New book wizard
// asks for, so the wizard window is only a screen over it.

mod data;
mod isbn;
mod make;
mod paints;
mod paths;
mod spec;

pub use data::preset_styles;
pub use data::{kind_infos, KindInfo, TrimChoice};
pub use isbn::is_valid_isbn13;
pub use make::{new_book, new_book_with_ids, NewBook};
pub use paints::{paint_presets, GradientPreset, PaintPresets, Swatch};
pub use paths::suggest_save_path;
pub use spec::{Look, NewBookSpec, Paper, PresetError, Words};
