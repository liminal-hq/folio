// `META-INF/manifest.json`: every file in the package, its type, its hash and its role
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};

/// The package's media type, stored uncompressed as the first entry so `file(1)` can sniff it.
pub const MIMETYPE: &str = "application/vnd.liminal.folio+zip";
/// The manifest's `format`.
pub const FORMAT: &str = "folio";
/// The container version: the zip rules, not the document schema. It changes rarely.
pub const FORMAT_VERSION: &str = "1.0";

/// What a file in the package is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileRole {
    Book,
    Styles,
    Masters,
    Settings,
    Text,
    Image,
    Font,
    Icc,
    Thumbnail,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestFile {
    pub path: String,
    pub media_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    pub role: FileRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format: String,
    pub format_version: String,
    pub generator: String,
    pub created: String,
    pub modified: String,
    pub files: Vec<ManifestFile>,
}

/// The role and media type of a file, from where it sits in the package.
pub(crate) fn classify(path: &str) -> (FileRole, &'static str) {
    let extension = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match path {
        "book.json" => (FileRole::Book, "application/json"),
        "styles.json" => (FileRole::Styles, "application/json"),
        "masters.json" => (FileRole::Masters, "application/json"),
        "settings.json" => (FileRole::Settings, "application/json"),
        _ if path.starts_with("text/") => (FileRole::Text, "text/markdown"),
        _ if path.starts_with("assets/images/") => (FileRole::Image, image_type(&extension)),
        _ if path.starts_with("assets/fonts/") => (FileRole::Font, font_type(&extension)),
        _ if path.starts_with("assets/icc/") => (FileRole::Icc, "application/vnd.iccprofile"),
        _ if path.starts_with("thumbnails/") => (FileRole::Thumbnail, image_type(&extension)),
        _ => (FileRole::Other, "application/octet-stream"),
    }
}

fn font_type(extension: &str) -> &'static str {
    match extension {
        "otf" => "font/otf",
        "ttc" | "otc" => "font/collection",
        _ => "font/ttf",
    }
}

fn image_type(extension: &str) -> &'static str {
    match extension {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "tif" | "tiff" => "image/tiff",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_are_classified_by_where_they_sit() {
        assert_eq!(classify("book.json"), (FileRole::Book, "application/json"));
        assert_eq!(
            classify("text/flow-main.md"),
            (FileRole::Text, "text/markdown")
        );
        assert_eq!(
            classify("assets/images/9f2c.JPG"),
            (FileRole::Image, "image/jpeg")
        );
        assert_eq!(
            classify("thumbnails/cover.png"),
            (FileRole::Thumbnail, "image/png")
        );
        assert_eq!(classify("whatever.bin").0, FileRole::Other);
    }

    #[test]
    fn the_manifest_uses_camel_case_keys() {
        let manifest = Manifest {
            format: FORMAT.into(),
            format_version: FORMAT_VERSION.into(),
            generator: "Folio test".into(),
            created: "2026-10-13T14:21:00Z".into(),
            modified: "2026-10-13T14:21:00Z".into(),
            files: vec![ManifestFile {
                path: "book.json".into(),
                media_type: "application/json".into(),
                sha256: None,
                role: FileRole::Book,
            }],
        };
        let json = serde_json::to_value(&manifest).unwrap();
        assert_eq!(json["formatVersion"], "1.0");
        assert_eq!(json["files"][0]["mediaType"], "application/json");
        assert!(json["files"][0].get("sha256").is_none());
    }
}
