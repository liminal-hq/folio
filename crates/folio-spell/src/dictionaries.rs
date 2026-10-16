// The dictionaries that can be downloaded, which one a book's language wants, and where they live
//
// Each dictionary is pinned by version, and each of its files by size and SHA-256, in
// `data/dictionaries.json`. A download is checked file by file and put in place only whole, so a
// dictionary on disk is always one that was checked (D-25).
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{DictionaryError, SpellbookChecker};

#[derive(Debug, Clone, Deserialize)]
pub struct DictionaryFile {
    /// What the file is kept as: `aff`, `dic` or `LICENSE`.
    pub name: String,
    pub sha256: String,
    pub bytes: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DictionaryInfo {
    /// The id the dictionary is kept under, and asked for by.
    pub id: String,
    pub name: String,
    pub package: String,
    pub version: String,
    /// The dictionary's own licence, as its package says. Its text is kept beside it.
    pub licence: String,
    /// The languages it is the dictionary for: BCP 47 codes, the most particular first.
    pub languages: Vec<String>,
    pub files: Vec<DictionaryFile>,
}

impl DictionaryInfo {
    /// The size of everything to download, in bytes.
    pub fn bytes(&self) -> u64 {
        self.files.iter().map(|f| f.bytes).sum()
    }
}

/// The id of the dictionary that comes with Folio.
pub const BUNDLED_ID: &str = "en-CA";

/// The dictionaries that can be downloaded.
pub fn catalogue() -> &'static [DictionaryInfo] {
    static CATALOGUE: OnceLock<Vec<DictionaryInfo>> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        serde_json::from_str(include_str!("../data/dictionaries.json"))
            .expect("the dictionary catalogue is valid")
    })
}

/// Which dictionary a language is checked with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    /// The English (Canada) dictionary that comes with Folio.
    Bundled,
    /// A downloaded one, by id.
    Installed(String),
    /// None: the language has no dictionary here.
    None,
}

fn primary(language: &str) -> &str {
    language.split(['-', '_']).next().unwrap_or("")
}

/// The dictionary for `language` among those `installed`: one made for that language exactly, then
/// one made for its main language (`fr` for `fr-CH`), then the bundled English (Canada) for any
/// English. A book in a language with none is not checked.
pub fn pick(language: &str, installed: &[String]) -> Pick {
    let language = language.replace('_', "-");
    if language.eq_ignore_ascii_case("en-CA") {
        return Pick::Bundled;
    }
    let made_for = |code: &str| {
        catalogue()
            .iter()
            .filter(|d| installed.contains(&d.id))
            .find(|d| d.languages.iter().any(|l| l.eq_ignore_ascii_case(code)))
    };
    if let Some(found) = made_for(&language).or_else(|| made_for(primary(&language))) {
        return Pick::Installed(found.id.clone());
    }
    if primary(&language).eq_ignore_ascii_case("en") {
        Pick::Bundled
    } else {
        Pick::None
    }
}

/// The dictionary a language would be best checked with if it were downloaded, when it is not
/// already: what to offer.
pub fn suggest_download(language: &str, installed: &[String]) -> Option<&'static DictionaryInfo> {
    let language = language.replace('_', "-");
    let best = |code: &str| {
        catalogue()
            .iter()
            .find(|d| d.languages.iter().any(|l| l.eq_ignore_ascii_case(code)))
    };
    let wanted = best(&language).or_else(|| best(primary(&language)))?;
    (!installed.contains(&wanted.id) && !language.eq_ignore_ascii_case("en-CA")).then_some(wanted)
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("there is no dictionary called `{0}`")]
    Unknown(String),
    #[error("the dictionary could not be downloaded: {0}")]
    Download(String),
    #[error("a file of the dictionary is not the one expected: {0}")]
    Mismatch(String),
    #[error("the dictionary could not be stored: {0}")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Dictionary(#[from] DictionaryError),
}

/// Gets the bytes at an address, taking no more than the given number of them.
pub type Fetch<'a> = &'a dyn Fn(&str, u64) -> Result<Vec<u8>, String>;

/// A folder of downloaded dictionaries, one folder each.
pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn info(id: &str) -> Result<&'static DictionaryInfo, StoreError> {
        catalogue()
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| StoreError::Unknown(id.to_owned()))
    }

    fn folder(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }

    /// The ids of the dictionaries that are fully in place.
    pub fn installed(&self) -> Vec<String> {
        catalogue()
            .iter()
            .filter(|d| {
                d.files
                    .iter()
                    .all(|f| self.folder(&d.id).join(&f.name).is_file())
            })
            .map(|d| d.id.clone())
            .collect()
    }

    /// A checker over an installed dictionary.
    pub fn load(&self, id: &str) -> Result<SpellbookChecker, StoreError> {
        let info = Self::info(id)?;
        let read = |name: &str| fs::read_to_string(self.folder(&info.id).join(name));
        Ok(SpellbookChecker::from_files(&read("aff")?, &read("dic")?)?)
    }

    /// Downloads a dictionary with `fetch` (which gets each file's address and the most bytes to
    /// take), checks every file against its pinned size and hash, and puts the dictionary in place
    /// only when all of it is right.
    pub fn install(&self, id: &str, fetch: Fetch<'_>) -> Result<(), StoreError> {
        self.install_info(Self::info(id)?, fetch)
    }

    fn install_info(&self, info: &DictionaryInfo, fetch: Fetch<'_>) -> Result<(), StoreError> {
        fs::create_dir_all(&self.root)?;
        let temporary = self.root.join(format!(".{}.part", info.id));
        let _ = fs::remove_dir_all(&temporary);
        fs::create_dir_all(&temporary)?;
        let result = (|| {
            for file in &info.files {
                let bytes = fetch(&file.url, file.bytes).map_err(StoreError::Download)?;
                if bytes.len() as u64 != file.bytes {
                    return Err(StoreError::Mismatch(format!(
                        "{} is {} bytes, not {}",
                        file.name,
                        bytes.len(),
                        file.bytes
                    )));
                }
                let hash = format!("{:x}", Sha256::digest(&bytes));
                if !hash.eq_ignore_ascii_case(&file.sha256) {
                    return Err(StoreError::Mismatch(format!(
                        "{} has the wrong hash",
                        file.name
                    )));
                }
                fs::write(temporary.join(&file.name), bytes)?;
            }
            // The dictionary is read once before it is kept, so one that cannot be read is never in place.
            let read = |name: &str| fs::read_to_string(temporary.join(name));
            SpellbookChecker::from_files(&read("aff")?, &read("dic")?)?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                let target = self.folder(&info.id);
                let _ = fs::remove_dir_all(&target);
                fs::rename(&temporary, &target)?;
                Ok(())
            }
            Err(error) => {
                let _ = fs::remove_dir_all(&temporary);
                Err(error)
            }
        }
    }

    /// Takes a downloaded dictionary away. Nothing happens when it is not there.
    pub fn remove(&self, id: &str) -> Result<(), StoreError> {
        let info = Self::info(id)?;
        match fs::remove_dir_all(self.folder(&info.id)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalogue_entry_is_pinned_and_safe() {
        let mut ids = std::collections::HashSet::new();
        for d in catalogue() {
            assert!(ids.insert(&d.id), "{} twice", d.id);
            assert!(
                d.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
                "{} is not a safe folder name",
                d.id
            );
            assert_ne!(d.id, BUNDLED_ID);
            let names: Vec<_> = d.files.iter().map(|f| f.name.as_str()).collect();
            assert_eq!(names, ["aff", "dic", "LICENSE"], "{}", d.id);
            for f in &d.files {
                assert_eq!(f.sha256.len(), 64, "{} {}", d.id, f.name);
                assert!(
                    f.url.starts_with("https://cdn.jsdelivr.net/npm/"),
                    "{}",
                    f.url
                );
                assert!(f.url.contains(&format!("@{}/", d.version)), "{}", f.url);
                assert!(f.bytes > 0 && f.bytes < 20 * 1024 * 1024);
            }
        }
    }

    #[test]
    fn a_language_picks_its_own_dictionary_then_its_main_languages_then_english() {
        let none: Vec<String> = Vec::new();
        let all: Vec<String> = catalogue().iter().map(|d| d.id.clone()).collect();
        assert_eq!(pick("en-CA", &all), Pick::Bundled);
        assert_eq!(pick("en_ca", &all), Pick::Bundled);
        assert_eq!(pick("en-GB", &all), Pick::Installed("en-GB".into()));
        assert_eq!(pick("en-GB", &none), Pick::Bundled);
        assert_eq!(pick("en", &all), Pick::Installed("en-US".into()));
        assert_eq!(pick("en-NZ", &all), Pick::Installed("en-US".into()));
        assert_eq!(pick("fr-CA", &all), Pick::Installed("fr".into()));
        assert_eq!(pick("fr-CH", &all), Pick::Installed("fr".into()));
        assert_eq!(pick("fr-CA", &none), Pick::None);
        assert_eq!(pick("ja", &all), Pick::None);
        assert_eq!(pick("", &all), Pick::None);
    }

    #[test]
    fn a_dictionary_is_offered_for_a_language_that_has_none_installed() {
        let none: Vec<String> = Vec::new();
        assert_eq!(suggest_download("fr-CA", &none).unwrap().id, "fr");
        assert_eq!(suggest_download("en-GB", &none).unwrap().id, "en-GB");
        assert!(suggest_download("fr-CA", &["fr".into()]).is_none());
        assert!(suggest_download("en-CA", &none).is_none());
        assert!(suggest_download("ja", &none).is_none());
    }

    /// A dictionary made to fit a catalogue entry, with the sizes and hashes of what it holds.
    fn fake_fetch(info: &DictionaryInfo) -> impl Fn(&str, u64) -> Result<Vec<u8>, String> + '_ {
        move |url, _| {
            // The real files are large, so a fetch hands back a small file of the right name; the
            // test below pins hashes to it by building its own entry.
            Err(format!("no network for {url} of {}", info.id))
        }
    }

    #[test]
    fn a_failed_download_leaves_nothing_behind() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        let info = &catalogue()[0];
        let err = store.install(&info.id, &fake_fetch(info)).unwrap_err();
        assert!(matches!(err, StoreError::Download(_)), "{err}");
        assert!(store.installed().is_empty());
        let leftovers: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }

    #[test]
    fn a_file_that_is_not_the_pinned_one_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        let info = &catalogue()[0];
        // Right size, wrong bytes.
        let wrong = |url: &str, _: u64| {
            let size = info.files.iter().find(|f| f.url == url).unwrap().bytes;
            Ok(vec![b'x'; size as usize])
        };
        let err = store.install(&info.id, &wrong).unwrap_err();
        assert!(matches!(err, StoreError::Mismatch(_)), "{err}");
        // Wrong size.
        let short = |_: &str, _: u64| Ok(vec![0u8; 3]);
        assert!(matches!(
            store.install(&info.id, &short).unwrap_err(),
            StoreError::Mismatch(_)
        ));
        assert!(store.installed().is_empty());
        assert!(fs::read_dir(dir.path()).unwrap().next().is_none());
    }

    #[test]
    fn a_dictionary_whose_files_are_the_pinned_ones_is_put_in_place_whole_and_can_be_used() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        let files: [(&str, &[u8]); 3] = [
            ("aff", b"SET UTF-8\n"),
            ("dic", b"2\nhello\nworld\n"),
            ("LICENSE", b"free to use"),
        ];
        let mut info = catalogue()[0].clone();
        info.files = files
            .iter()
            .map(|(name, bytes)| DictionaryFile {
                name: (*name).into(),
                sha256: format!("{:x}", Sha256::digest(bytes)),
                bytes: bytes.len() as u64,
                url: format!("https://example.invalid/{name}"),
            })
            .collect();
        let fetch = |url: &str, _: u64| {
            files
                .iter()
                .find(|(name, _)| url.ends_with(*name))
                .map(|(_, bytes)| bytes.to_vec())
                .ok_or_else(|| "not there".to_owned())
        };
        store.install_info(&info, &fetch).unwrap();
        // It counts as installed only as the catalogue names it, which this entry's id does.
        assert_eq!(store.installed(), [info.id.clone()]);
        let checker = {
            let read =
                |name: &str| fs::read_to_string(dir.path().join(&info.id).join(name)).unwrap();
            SpellbookChecker::from_files(&read("aff"), &read("dic")).unwrap()
        };
        assert!(crate::Checker::check(&checker, "hello"));
        assert!(!crate::Checker::check(&checker, "helo"));
        store.remove(&info.id).unwrap();
        assert!(store.installed().is_empty());
    }

    #[test]
    fn an_unknown_id_is_refused_before_anything_is_touched() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        for id in ["nope", "../x", ""] {
            assert!(matches!(
                store.install(id, &|_, _| Ok(vec![])),
                Err(StoreError::Unknown(_))
            ));
            assert!(matches!(store.remove(id), Err(StoreError::Unknown(_))));
            assert!(matches!(store.load(id), Err(StoreError::Unknown(_))));
        }
        assert!(fs::read_dir(dir.path()).unwrap().next().is_none());
    }

    #[test]
    fn removing_what_is_not_there_is_fine() {
        let dir = tempfile::tempdir().unwrap();
        Store::new(dir.path()).remove("fr").unwrap();
    }
}
