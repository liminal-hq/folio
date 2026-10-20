// The package in memory, and how it is opened and saved
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use chrono::{SecondsFormat, Utc};
use folio_model::Book;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::error::PackageError;
use crate::manifest::{
    classify, FileRole, Manifest, ManifestFile, FORMAT, FORMAT_VERSION, MIMETYPE,
};

/// The newest `book.json` schema this build reads and writes.
pub const SUPPORTED_SCHEMA: u32 = 1;

const MANIFEST_PATH: &str = "META-INF/manifest.json";
const MIMETYPE_PATH: &str = "mimetype";

/// Something stored under `assets/` or `thumbnails/`: in memory, or still in the file it came from.
#[derive(Debug, Clone)]
enum Blob {
    Memory(Vec<u8>),
    /// In the zip this package was opened from or last saved to; read and copied on demand.
    Packed,
}

/// How a save is done.
#[derive(Debug, Clone)]
pub struct SaveOptions {
    /// Written into the manifest, e.g. `Folio 0.1.0 (Liminal HQ)`.
    pub generator: String,
    /// Keep the previous save as `name.folio~`.
    pub keep_backup: bool,
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            generator: "Folio (Liminal HQ)".into(),
            keep_backup: true,
        }
    }
}

/// A book and everything saved with it.
#[derive(Debug)]
pub struct Package {
    /// The file this package was opened from or last saved to.
    source: Option<PathBuf>,
    created: String,
    book: Book,
    styles: Value,
    masters: Value,
    settings: Value,
    /// Long text, keyed by package path (`text/flow-main.md`).
    flows: BTreeMap<String, String>,
    /// Pictures, fonts, colour profiles and thumbnails.
    assets: BTreeMap<String, Blob>,
    /// Files this version does not know, kept so a save does not lose them.
    other: BTreeMap<String, Vec<u8>>,
    /// Hashes the file's manifest gave for entries copied across a save without being read.
    hashes: BTreeMap<String, String>,
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn pretty(value: &impl serde::Serialize, file: &str) -> Result<Vec<u8>, PackageError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| PackageError::Json {
        file: file.into(),
        message: e.to_string(),
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut out, byte| {
            out.push_str(&format!("{byte:02x}"));
            out
        })
}

/// A path that stays inside the package: relative, no `..`, no backslashes or empty parts.
fn safe(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

impl Package {
    /// A new package around `book`, not yet saved.
    pub fn new(book: Book) -> Self {
        Self {
            source: None,
            created: now(),
            book,
            styles: json!({}),
            masters: json!({}),
            settings: json!({}),
            flows: BTreeMap::new(),
            assets: BTreeMap::new(),
            other: BTreeMap::new(),
            hashes: BTreeMap::new(),
        }
    }

    pub fn book(&self) -> &Book {
        &self.book
    }

    pub fn book_mut(&mut self) -> &mut Book {
        &mut self.book
    }

    /// `styles.json`: paragraph and character styles, and colour swatches.
    pub fn styles(&self) -> &Value {
        &self.styles
    }

    pub fn styles_mut(&mut self) -> &mut Value {
        &mut self.styles
    }

    /// `masters.json`: master pages.
    pub fn masters(&self) -> &Value {
        &self.masters
    }

    pub fn masters_mut(&mut self) -> &mut Value {
        &mut self.masters
    }

    /// `settings.json`: per-book export settings, the last target and preflight ignores.
    pub fn settings(&self) -> &Value {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut Value {
        &mut self.settings
    }

    /// The file this package was opened from or last saved to.
    pub fn path(&self) -> Option<&Path> {
        self.source.as_deref()
    }

    /// The text of a flow, by package path (`text/flow-main.md`).
    pub fn flow_text(&self, path: &str) -> Option<&str> {
        self.flows.get(path).map(String::as_str)
    }

    /// Sets the text of a flow. The path must be `text/…​.md`.
    pub fn set_flow_text(
        &mut self,
        path: &str,
        text: impl Into<String>,
    ) -> Result<(), PackageError> {
        if !safe(path) || !path.starts_with("text/") || !path.ends_with(".md") {
            return Err(PackageError::BadPath(path.into()));
        }
        self.flows.insert(path.into(), text.into());
        Ok(())
    }

    /// The paths of every flow.
    pub fn flow_paths(&self) -> impl Iterator<Item = &str> {
        self.flows.keys().map(String::as_str)
    }

    /// A file this version does not know about, kept from the file that was opened.
    pub fn extra_file(&self, path: &str) -> Option<&[u8]> {
        self.other.get(path).map(Vec::as_slice)
    }

    /// Adds a picture under its content hash and returns its package path
    /// (`assets/images/<sha256>.<ext>`). The same bytes always give the same path, so adding a
    /// picture twice stores it once.
    pub fn add_image(&mut self, bytes: Vec<u8>, extension: &str) -> String {
        let extension: String = extension
            .trim_start_matches('.')
            .to_ascii_lowercase()
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .collect();
        let extension = if extension.is_empty() {
            "bin".to_owned()
        } else {
            extension
        };
        let path = format!("assets/images/{}.{extension}", sha256(&bytes));
        self.assets
            .entry(path.clone())
            .or_insert(Blob::Memory(bytes));
        self.hashes.remove(&path);
        path
    }

    /// Adds a font file, named by its hash so the same file is kept once, and returns its path.
    /// `extension` is `ttf`, `otf`, `ttc` or `otc`; anything else is called `ttf`.
    pub fn add_font(&mut self, bytes: Vec<u8>, extension: &str) -> String {
        let extension = extension.trim_start_matches('.').to_ascii_lowercase();
        let extension = match extension.as_str() {
            "otf" | "ttc" | "otc" => extension.as_str(),
            _ => "ttf",
        };
        let path = format!("assets/fonts/{}.{extension}", sha256(&bytes));
        self.assets
            .entry(path.clone())
            .or_insert(Blob::Memory(bytes));
        self.hashes.remove(&path);
        path
    }

    /// Takes an asset out of the package, such as a font the book no longer keeps. It is gone from
    /// the file at the next save.
    pub fn remove_asset(&mut self, path: &str) -> bool {
        self.hashes.remove(path);
        self.assets.remove(path).is_some()
    }

    /// The paths of every picture, font, colour profile and thumbnail.
    pub fn asset_paths(&self) -> impl Iterator<Item = &str> {
        self.assets.keys().map(String::as_str)
    }

    /// The bytes of an asset, read from the file when it has not been loaded.
    pub fn asset_bytes(&self, path: &str) -> Result<Vec<u8>, PackageError> {
        match self.assets.get(path) {
            None => Err(PackageError::Missing(path.into())),
            Some(Blob::Memory(bytes)) => Ok(bytes.clone()),
            Some(Blob::Packed) => {
                let source = self
                    .source
                    .as_ref()
                    .ok_or_else(|| PackageError::Missing(path.into()))?;
                let mut archive = open_archive(source)?;
                let mut entry = archive
                    .by_name(path)
                    .map_err(|_| PackageError::Missing(path.into()))?;
                let mut bytes = Vec::new();
                entry
                    .read_to_end(&mut bytes)
                    .map_err(|e| PackageError::io(source, e))?;
                Ok(bytes)
            }
        }
    }

    /// Opens a `.folio` file.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, PackageError> {
        let path = path.as_ref();
        let mut archive = open_archive(path)?;

        // The first entry is `mimetype`, stored, holding exactly the media type.
        {
            let mut first = archive
                .by_index(0)
                .map_err(|_| PackageError::NotFolio("the file is empty".into()))?;
            if first.name() != MIMETYPE_PATH {
                return Err(PackageError::NotFolio(
                    "it has no `mimetype` entry first".into(),
                ));
            }
            let mut media = String::new();
            first
                .read_to_string(&mut media)
                .map_err(|e| PackageError::io(path, e))?;
            if media != MIMETYPE {
                return Err(PackageError::NotFolio(format!(
                    "its media type is `{media}`"
                )));
            }
        }

        let manifest: Manifest = read_json(&mut archive, MANIFEST_PATH, path)?
            .ok_or_else(|| PackageError::Missing(MANIFEST_PATH.into()))
            .and_then(|value| {
                serde_json::from_value(value).map_err(|e| PackageError::Json {
                    file: MANIFEST_PATH.into(),
                    message: e.to_string(),
                })
            })?;
        if manifest.format != FORMAT {
            return Err(PackageError::NotFolio(format!(
                "its format is `{}`",
                manifest.format
            )));
        }
        if manifest.format_version.split('.').next() != FORMAT_VERSION.split('.').next() {
            return Err(PackageError::NewerFormat {
                found: manifest.format_version,
                supported: FORMAT_VERSION.into(),
            });
        }

        let book_value = read_json(&mut archive, "book.json", path)?
            .ok_or_else(|| PackageError::Missing("book.json".into()))?;
        let schema = book_value
            .get("schema")
            .and_then(Value::as_u64)
            .ok_or_else(|| PackageError::Json {
                file: "book.json".into(),
                message: "it has no `schema`".into(),
            })? as u32;
        if schema > SUPPORTED_SCHEMA {
            return Err(PackageError::NewerSchema {
                found: schema,
                supported: SUPPORTED_SCHEMA,
            });
        }
        let book: Book = serde_json::from_value(book_value).map_err(|e| PackageError::Json {
            file: "book.json".into(),
            message: e.to_string(),
        })?;

        let mut package = Package {
            source: Some(path.to_path_buf()),
            created: manifest.created.clone(),
            book,
            styles: read_json(&mut archive, "styles.json", path)?.unwrap_or_else(|| json!({})),
            masters: read_json(&mut archive, "masters.json", path)?.unwrap_or_else(|| json!({})),
            settings: read_json(&mut archive, "settings.json", path)?.unwrap_or_else(|| json!({})),
            flows: BTreeMap::new(),
            assets: BTreeMap::new(),
            other: BTreeMap::new(),
            hashes: manifest
                .files
                .iter()
                .filter_map(|file| file.sha256.clone().map(|hash| (file.path.clone(), hash)))
                .collect(),
        };

        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            if entry.is_dir() {
                continue;
            }
            let Some(name) = entry
                .enclosed_name()
                .and_then(|n| n.to_str().map(str::to_owned))
            else {
                return Err(PackageError::BadPath(entry.name().into()));
            };
            let name = name.replace('\\', "/");
            match name.as_str() {
                MIMETYPE_PATH | MANIFEST_PATH | "book.json" | "styles.json" | "masters.json"
                | "settings.json" => {}
                // The autosave journal is transient: it belongs to a session, not to the book.
                _ if name.starts_with("recovery/") => {}
                _ if name.starts_with("text/") && name.ends_with(".md") => {
                    let mut text = String::new();
                    entry
                        .read_to_string(&mut text)
                        .map_err(|e| PackageError::io(path, e))?;
                    package.flows.insert(name, text);
                }
                _ if name.starts_with("assets/") || name.starts_with("thumbnails/") => {
                    package.assets.insert(name, Blob::Packed);
                }
                _ => {
                    let mut bytes = Vec::new();
                    entry
                        .read_to_end(&mut bytes)
                        .map_err(|e| PackageError::io(path, e))?;
                    package.other.insert(name, bytes);
                }
            }
        }
        Ok(package)
    }

    /// Saves to `path`, atomically. The new file is written beside the old one and renamed over it;
    /// with `keep_backup`, what was there is kept as `name.folio~`. A failed save leaves the old
    /// file as it was. After a save the package is the file at `path`.
    pub fn save(
        &mut self,
        path: impl AsRef<Path>,
        options: &SaveOptions,
    ) -> Result<(), PackageError> {
        let path = path.as_ref();
        self.write_atomically(path, options)?;
        // Everything is in the new file now, so what was in memory can be read from there.
        let absolute = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        for blob in self.assets.values_mut() {
            *blob = Blob::Packed;
        }
        self.source = Some(absolute);
        Ok(())
    }

    /// Writes the book as it is to `path`, atomically, and leaves the package as it was: it is not
    /// the file at `path` afterwards, so what it reads still comes from where it did. For copies
    /// kept beside the real file, such as the ones that let a crash be recovered from.
    pub fn save_copy(
        &mut self,
        path: impl AsRef<Path>,
        options: &SaveOptions,
    ) -> Result<(), PackageError> {
        self.write_atomically(path.as_ref(), options)
    }

    fn write_atomically(&mut self, path: &Path, options: &SaveOptions) -> Result<(), PackageError> {
        let temporary = beside(path, ".tmp");
        let result = self.write_to(&temporary, options).and_then(|()| {
            if options.keep_backup && path.exists() {
                let backup = beside(path, "~");
                let _ = fs::remove_file(&backup);
                // A hard link keeps the old file whole until the rename; copy where links are not allowed.
                if fs::hard_link(path, &backup).is_err() {
                    fs::copy(path, &backup).map_err(|e| PackageError::io(&backup, e))?;
                }
            }
            fs::rename(&temporary, path).map_err(|e| PackageError::io(path, e))
        });
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    fn write_to(&mut self, temporary: &Path, options: &SaveOptions) -> Result<(), PackageError> {
        let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        let book = pretty(&self.book, "book.json")?;
        let styles = pretty(&self.styles, "styles.json")?;
        let masters = pretty(&self.masters, "masters.json")?;
        let settings = pretty(&self.settings, "settings.json")?;

        // The manifest lists every file with its hash. Anything copied across without being read
        // keeps the hash the old manifest gave it.
        let mut files: Vec<ManifestFile> = Vec::new();
        let mut list = |path: &str, hash: Option<String>| {
            let (role, media_type) = classify(path);
            files.push(ManifestFile {
                path: path.into(),
                media_type: media_type.into(),
                sha256: hash,
                role,
            });
        };
        list("book.json", Some(sha256(&book)));
        list("styles.json", Some(sha256(&styles)));
        list("masters.json", Some(sha256(&masters)));
        list("settings.json", Some(sha256(&settings)));
        for (path, text) in &self.flows {
            list(path, Some(sha256(text.as_bytes())));
        }
        for (path, blob) in &self.assets {
            let hash = match blob {
                Blob::Memory(bytes) => Some(sha256(bytes)),
                Blob::Packed => self.hashes.get(path).cloned(),
            };
            list(path, hash);
        }
        for (path, bytes) in &self.other {
            list(path, Some(sha256(bytes)));
        }
        let manifest = Manifest {
            format: FORMAT.into(),
            format_version: FORMAT_VERSION.into(),
            generator: options.generator.clone(),
            created: self.created.clone(),
            modified: now(),
            files,
        };
        let manifest_bytes = pretty(&manifest, MANIFEST_PATH)?;

        let mut source = match (
            &self.source,
            self.assets.values().any(|b| matches!(b, Blob::Packed)),
        ) {
            (Some(source), true) => Some(open_archive(source)?),
            _ => None,
        };

        let file = File::create(temporary).map_err(|e| PackageError::io(temporary, e))?;
        let mut zip = ZipWriter::new(file);
        let io = |e: std::io::Error| PackageError::io(temporary, e);

        zip.start_file(MIMETYPE_PATH, stored)?;
        zip.write_all(MIMETYPE.as_bytes()).map_err(io)?;
        for (name, bytes) in [
            (MANIFEST_PATH, &manifest_bytes),
            ("book.json", &book),
            ("styles.json", &styles),
            ("masters.json", &masters),
            ("settings.json", &settings),
        ] {
            zip.start_file(name, deflated)?;
            zip.write_all(bytes).map_err(io)?;
        }
        for (name, text) in &self.flows {
            zip.start_file(name, deflated)?;
            zip.write_all(text.as_bytes()).map_err(io)?;
        }
        for (name, blob) in &self.assets {
            match blob {
                Blob::Memory(bytes) => {
                    // Pictures are already compressed; deflating them again only costs time.
                    let incompressible =
                        matches!(classify(name).0, FileRole::Image | FileRole::Thumbnail)
                            && !name.ends_with(".svg");
                    zip.start_file(name, if incompressible { stored } else { deflated })?;
                    zip.write_all(bytes).map_err(io)?;
                }
                Blob::Packed => {
                    let archive = source
                        .as_mut()
                        .ok_or_else(|| PackageError::Missing(name.clone()))?;
                    let index = archive
                        .index_for_name(name)
                        .ok_or_else(|| PackageError::Missing(name.clone()))?;
                    zip.raw_copy_file(archive.by_index_raw(index)?)?;
                }
            }
        }
        for (name, bytes) in &self.other {
            zip.start_file(name, deflated)?;
            zip.write_all(bytes).map_err(io)?;
        }
        let file = zip.finish()?;
        file.sync_all().map_err(io)?;
        Ok(())
    }
}

/// `name.folio` with `suffix` after it: the temporary file and the backup.
fn beside(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

fn open_archive(path: &Path) -> Result<ZipArchive<File>, PackageError> {
    let file = File::open(path).map_err(|e| PackageError::io(path, e))?;
    Ok(ZipArchive::new(file)?)
}

fn read_json(
    archive: &mut ZipArchive<File>,
    name: &str,
    path: &Path,
) -> Result<Option<Value>, PackageError> {
    let mut entry = match archive.by_name(name) {
        Ok(entry) => entry,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .map_err(|e| PackageError::io(path, e))?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|e| PackageError::Json {
            file: name.into(),
            message: e.to_string(),
        })
}
