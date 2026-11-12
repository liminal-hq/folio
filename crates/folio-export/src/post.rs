// What krilla cannot write, put into a finished PDF
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A viewer shows a file's name in its title bar unless the file says to show its title instead,
//! which a screen reader also needs to say what it has opened. krilla only writes that key for
//! PDF/UA, so a tagged file is opened again and given it.

use lopdf::{Dictionary, Document, Object};

use crate::error::ExportError;

/// Makes the catalogue of `doc` ask a viewer to show the document's title.
pub fn display_doc_title(doc: &mut Document) -> Result<(), ExportError> {
    let fail = |e: lopdf::Error| ExportError::Pdf(e.to_string());
    let root_id = doc
        .trailer
        .get(b"Root")
        .and_then(Object::as_reference)
        .map_err(fail)?;
    // The preferences may be written in the catalogue or as an object of their own.
    let existing = doc
        .get_object(root_id)
        .and_then(Object::as_dict)
        .map_err(fail)?
        .get(b"ViewerPreferences")
        .ok()
        .cloned();
    match existing {
        Some(Object::Reference(id)) => {
            doc.get_object_mut(id)
                .and_then(Object::as_dict_mut)
                .map_err(fail)?
                .set("DisplayDocTitle", true);
        }
        Some(Object::Dictionary(mut preferences)) => {
            preferences.set("DisplayDocTitle", true);
            doc.get_object_mut(root_id)
                .and_then(Object::as_dict_mut)
                .map_err(fail)?
                .set("ViewerPreferences", Object::Dictionary(preferences));
        }
        _ => {
            let mut preferences = Dictionary::new();
            preferences.set("DisplayDocTitle", true);
            doc.get_object_mut(root_id)
                .and_then(Object::as_dict_mut)
                .map_err(fail)?
                .set("ViewerPreferences", Object::Dictionary(preferences));
        }
    }
    Ok(())
}

/// `bytes` with the document's title asked for in a viewer's title bar.
pub fn with_display_doc_title(bytes: &[u8]) -> Result<Vec<u8>, ExportError> {
    let mut doc = Document::load_mem(bytes).map_err(|e| ExportError::Pdf(e.to_string()))?;
    display_doc_title(&mut doc)?;
    let mut out = Vec::new();
    doc.save_to(&mut out)
        .map_err(|e| ExportError::Pdf(e.to_string()))?;
    Ok(out)
}
