// The dictionaries the plugin checks with: the one that comes with Folio, and those downloaded
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::cell::Cell;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use folio_spell::dictionaries::{catalogue, pick, suggest_download, Pick, Store, BUNDLED_ID};
use folio_spell::SpellbookChecker;

use crate::error::Error;
use crate::wire::{DictionaryOffer, DictionaryStatus};

/// The most a downloaded file may be, whatever the catalogue says, in bytes.
const MAX_DOWNLOAD: u64 = 20 * 1024 * 1024;

/// How the dictionaries are found, kept in memory once read, and downloaded.
#[derive(Default)]
pub struct Spelling {
    bundled: OnceLock<Option<Arc<SpellbookChecker>>>,
    loaded: Mutex<HashMap<String, Arc<SpellbookChecker>>>,
    /// Where downloaded dictionaries are kept.
    folder: Mutex<Option<PathBuf>>,
}

fn locked<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

impl Spelling {
    /// Keeps downloaded dictionaries in `folder`.
    pub fn keep_in(&self, folder: PathBuf) {
        *locked(&self.folder) = Some(folder);
    }

    fn store(&self) -> Result<Store, Error> {
        locked(&self.folder)
            .clone()
            .map(Store::new)
            .ok_or_else(|| Error::Dictionary("there is nowhere to keep dictionaries".into()))
    }

    fn installed(&self) -> Vec<String> {
        self.store().map(|s| s.installed()).unwrap_or_default()
    }

    fn bundled(&self) -> Option<Arc<SpellbookChecker>> {
        self.bundled
            .get_or_init(|| SpellbookChecker::en_ca().ok().map(Arc::new))
            .clone()
    }

    /// The checker for a book's language, and the id of the dictionary it is. `None` when no
    /// dictionary can check the language.
    pub fn for_language(&self, language: &str) -> Option<(String, Arc<SpellbookChecker>)> {
        match pick(language, &self.installed()) {
            Pick::Bundled => self.bundled().map(|c| (BUNDLED_ID.to_owned(), c)),
            Pick::Installed(id) => {
                if let Some(found) = locked(&self.loaded).get(&id) {
                    return Some((id, found.clone()));
                }
                let checker = Arc::new(self.store().ok()?.load(&id).ok()?);
                locked(&self.loaded).insert(id.clone(), checker.clone());
                Some((id, checker))
            }
            Pick::None => None,
        }
    }

    /// A better dictionary than the one in use, if there is one to download for the language.
    pub fn offer(&self, language: &str) -> Option<DictionaryOffer> {
        suggest_download(language, &self.installed()).map(|d| DictionaryOffer {
            id: d.id.clone(),
            name: d.name.clone(),
            bytes: d.bytes() as u32,
        })
    }

    /// Every dictionary there is, with whether it is in place.
    pub fn status(&self) -> Vec<DictionaryStatus> {
        let installed = self.installed();
        let mut out = vec![DictionaryStatus {
            id: BUNDLED_ID.to_owned(),
            name: "English (Canada)".to_owned(),
            languages: vec!["en-CA".to_owned()],
            licence: "(MIT AND BSD)".to_owned(),
            bytes: 0,
            installed: true,
            bundled: true,
        }];
        out.extend(catalogue().iter().map(|d| DictionaryStatus {
            id: d.id.clone(),
            name: d.name.clone(),
            languages: d.languages.clone(),
            licence: d.licence.clone(),
            bytes: d.bytes() as u32,
            installed: installed.contains(&d.id),
            bundled: false,
        }));
        out
    }

    /// Downloads a dictionary and checks it against what the catalogue pins. `progress` is told how
    /// many bytes have arrived of how many are expected, about once for each percent and at the end.
    pub fn download(&self, id: &str, progress: &dyn Fn(u64, u64)) -> Result<(), Error> {
        let total = catalogue()
            .iter()
            .find(|d| d.id == id)
            .map_or(0, |d| d.bytes());
        let received = Cell::new(0u64);
        let told = Cell::new(0u64);
        let step = (total / 100).max(1);
        let fetch = |url: &str, most: u64| -> Result<Vec<u8>, String> {
            let mut response = ureq::get(url).call().map_err(|e| e.to_string())?;
            let limit = most.min(MAX_DOWNLOAD) + 1;
            let mut reader = response.body_mut().with_config().limit(limit).reader();
            let mut bytes = Vec::new();
            let mut chunk = [0u8; 16 * 1024];
            loop {
                let n = reader.read(&mut chunk).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..n]);
                received.set(received.get() + n as u64);
                if received.get() - told.get() >= step {
                    told.set(received.get());
                    progress(received.get(), total);
                }
            }
            Ok(bytes)
        };
        self.store()?
            .install(id, &fetch)
            .map_err(|e| Error::Dictionary(e.to_string()))?;
        progress(total, total);
        locked(&self.loaded).remove(id);
        Ok(())
    }

    /// Takes a downloaded dictionary away.
    pub fn remove(&self, id: &str) -> Result<(), Error> {
        self.store()?
            .remove(id)
            .map_err(|e| Error::Dictionary(e.to_string()))?;
        locked(&self.loaded).remove(id);
        Ok(())
    }
}
