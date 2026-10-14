// Reads and writes `.folio` project files, the zip package described in spec/03-project-format.md
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// A `.folio` file is an ODF-style zip: a stored `mimetype` entry first, a manifest, the document as
// JSON, long text as CommonMark, and the original pictures under their content hash. Unzipping one
// always yields the person's words and pictures. Unknown files and unknown JSON keys survive a save,
// and a save is atomic: the new file is written beside the old one and renamed over it, with the
// previous save kept as `name.folio~`.
//
// This crate has no `tauri` dependency. Pictures are read lazily and copied across a save without
// being recompressed, so a book with many large pictures does not sit in memory.

mod error;
mod manifest;
mod package;

pub use error::PackageError;
pub use manifest::{FileRole, Manifest, ManifestFile, FORMAT, FORMAT_VERSION, MIMETYPE};
pub use package::{Package, SaveOptions, SUPPORTED_SCHEMA};
