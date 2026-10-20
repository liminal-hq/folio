// Making a PDF a PDF/X-4: the output intent, the identification and the keys a print shop checks
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `krilla` writes the pages, fonts and pictures but has no PDF/X mode (D-34), so the file it makes
//! is opened again and given what PDF/X-4 asks of the catalogue: an output intent that names the
//! colour condition and carries its ICC profile, `GTS_PDFXVersion` in the information and in the
//! XMP, `Trapped`, a document ID and the XMP packet itself.

use lopdf::{Dictionary, Document, Object, ObjectId, Stream, StringFormat};

use crate::error::ExportError;
use crate::time::utc_now;

/// The colour condition a file is made for.
pub struct OutputCondition<'a> {
    /// What the condition is called, as the registry names it (`FOGRA39`, `sRGB IEC61966-2.1`).
    pub identifier: &'a str,
    /// The ICC profile the colours are in.
    pub profile: &'a [u8],
    /// How many colour components the profile has: 3 for RGB, 4 for CMYK.
    pub components: u8,
}

fn text(value: &str) -> Object {
    Object::String(value.as_bytes().to_vec(), StringFormat::Literal)
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// A stable 32-digit id from the file's content, so the same book makes the same id.
fn id_of(bytes: &[u8], salt: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut out = String::new();
    for round in 0u8..2 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        (round, salt, bytes.len()).hash(&mut hasher);
        bytes.iter().step_by(97).for_each(|b| b.hash(&mut hasher));
        out.push_str(&format!("{:016x}", hasher.finish()));
    }
    out
}

/// `bytes` as a PDF/X-4 file for `condition`.
pub fn make_pdf_x4(
    bytes: &[u8],
    title: &str,
    condition: &OutputCondition<'_>,
) -> Result<Vec<u8>, ExportError> {
    let fail = |e: lopdf::Error| ExportError::Pdf(e.to_string());
    let mut doc = Document::load_mem(bytes).map_err(fail)?;
    let now = utc_now();

    // The profile, and the output intent that names it.
    let mut profile = Dictionary::new();
    profile.set("N", i64::from(condition.components));
    profile.set(
        "Alternate",
        Object::Name(
            if condition.components == 4 {
                "DeviceCMYK"
            } else {
                "DeviceRGB"
            }
            .into(),
        ),
    );
    let mut stream = Stream::new(profile, condition.profile.to_vec());
    let _ = stream.compress();
    let profile_id = doc.add_object(stream);
    let mut intent = Dictionary::new();
    intent.set("Type", Object::Name(b"OutputIntent".to_vec()));
    intent.set("S", Object::Name(b"GTS_PDFX".to_vec()));
    intent.set("OutputConditionIdentifier", text(condition.identifier));
    intent.set("OutputCondition", text(condition.identifier));
    intent.set("RegistryName", text("http://www.color.org"));
    intent.set("Info", text(condition.identifier));
    intent.set("DestOutputProfile", Object::Reference(profile_id));
    let intent_id = doc.add_object(intent);

    // The XMP packet.
    let document_id = id_of(bytes, "document");
    let instance_id = id_of(bytes, "instance");
    let xmp = format!(
        "<?xpacket begin=\"\u{feff}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n<x:xmpmeta xmlns:x=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"><rdf:Description rdf:about=\"\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\" xmlns:xmpMM=\"http://ns.adobe.com/xap/1.0/mm/\" xmlns:pdf=\"http://ns.adobe.com/pdf/1.3/\" xmlns:pdfxid=\"http://www.npes.org/pdfx/ns/id/\"><dc:format>application/pdf</dc:format><dc:title><rdf:Alt><rdf:li xml:lang=\"x-default\">{title}</rdf:li></rdf:Alt></dc:title><xmp:CreateDate>{now}</xmp:CreateDate><xmp:ModifyDate>{now}</xmp:ModifyDate><xmp:MetadataDate>{now}</xmp:MetadataDate><xmp:CreatorTool>Folio</xmp:CreatorTool><pdf:Producer>Folio</pdf:Producer><pdf:Trapped>False</pdf:Trapped><xmpMM:DocumentID>uuid:{document_id}</xmpMM:DocumentID><xmpMM:InstanceID>uuid:{instance_id}</xmpMM:InstanceID><pdfxid:GTS_PDFXVersion>PDF/X-4</pdfxid:GTS_PDFXVersion></rdf:Description></rdf:RDF></x:xmpmeta>\n<?xpacket end=\"w\"?>",
        title = escape(title),
    );
    let mut meta = Dictionary::new();
    meta.set("Type", Object::Name(b"Metadata".to_vec()));
    meta.set("Subtype", Object::Name(b"XML".to_vec()));
    let meta_id = doc.add_object(Stream::new(meta, xmp.into_bytes()));

    // The catalogue.
    let root_id = doc
        .trailer
        .get(b"Root")
        .and_then(Object::as_reference)
        .map_err(fail)?;
    {
        let root = doc
            .get_object_mut(root_id)
            .and_then(Object::as_dict_mut)
            .map_err(fail)?;
        root.set("OutputIntents", vec![Object::Reference(intent_id)]);
        root.set("Metadata", Object::Reference(meta_id));
    }

    // The information dictionary.
    let info_id: ObjectId = match doc.trailer.get(b"Info").and_then(Object::as_reference) {
        Ok(id) => id,
        Err(_) => {
            let id = doc.add_object(Dictionary::new());
            doc.trailer.set("Info", Object::Reference(id));
            id
        }
    };
    {
        let info = doc
            .get_object_mut(info_id)
            .and_then(Object::as_dict_mut)
            .map_err(fail)?;
        info.set("Title", text(title));
        info.set("Producer", text("Folio"));
        info.set("GTS_PDFXVersion", text("PDF/X-4"));
        info.set("Trapped", Object::Name(b"False".to_vec()));
        let stamp = format!("D:{}", now.replace(['-', ':', 'T', 'Z'], ""));
        info.set("CreationDate", text(&format!("{stamp}Z")));
        info.set("ModDate", text(&format!("{stamp}Z")));
    }

    // A document ID, which PDF/X asks for.
    if !doc.trailer.has(b"ID") {
        let first = Object::String(document_id.into_bytes(), StringFormat::Hexadecimal);
        let second = Object::String(instance_id.into_bytes(), StringFormat::Hexadecimal);
        doc.trailer.set("ID", vec![first, second]);
    }
    let mut out = Vec::new();
    doc.save_to(&mut out)
        .map_err(|e| ExportError::Pdf(e.to_string()))?;
    Ok(out)
}
