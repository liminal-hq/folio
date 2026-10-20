// Tests for reading an OpenDocument text file into a story
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::io::{Cursor, Write};

use folio_import::{import_odt, OdtError};

fn odt(body: &str) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut out);
        zip.start_file("content.xml", zip::write::SimpleFileOptions::default())
            .unwrap();
        write!(
            zip,
            "<office:document-content xmlns:office=\"o\" xmlns:text=\"t\" xmlns:style=\"s\" xmlns:fo=\"f\" xmlns:table=\"b\">\
             <office:automatic-styles>\
             <style:style style:name=\"T1\" style:family=\"text\"><style:text-properties fo:font-weight=\"bold\"/></style:style>\
             <style:style style:name=\"T2\" style:family=\"text\"><style:text-properties fo:font-style=\"italic\"/></style:style>\
             </office:automatic-styles><office:body><office:text>{body}</office:text></office:body></office:document-content>"
        )
        .unwrap();
        zip.finish().unwrap();
    }
    out.into_inner()
}

fn text(para: &folio_model::StoryPara) -> String {
    para.inline.iter().map(|r| r.text.as_str()).collect()
}

#[test]
fn headings_quotes_spans_and_lists_come_across() {
    let body = r#"
        <text:h text:outline-level="1">The kettle</text:h>
        <text:h text:outline-level="2">Later</text:h>
        <text:p>She <text:span text:style-name="T2">sang</text:span> and <text:span text:style-name="T1">boiled</text:span>.</text:p>
        <text:p text:style-name="Quotations">Boil it.</text:p>
        <text:list><text:list-item><text:p>Tea</text:p></text:list-item></text:list>
    "#;
    let imported = import_odt(&odt(body)).unwrap();
    let s = &imported.story;
    assert_eq!(s.len(), 5);
    assert_eq!(s[0].style.as_ref().map(|x| x.0.as_str()), Some("h1"));
    assert_eq!(s[1].style.as_ref().map(|x| x.0.as_str()), Some("h2"));
    assert_eq!(text(&s[2]), "She sang and boiled.");
    assert!(s[2].inline.iter().any(|r| r.italic && r.text == "sang"));
    assert!(s[2].inline.iter().any(|r| r.bold && r.text == "boiled"));
    assert_eq!(s[3].style.as_ref().map(|x| x.0.as_str()), Some("epigraph"));
    assert_eq!(text(&s[4]), "\u{2013} Tea");
    assert_eq!(imported.report.headings, 2);
}

#[test]
fn tables_pictures_and_notes_are_left_out_and_said() {
    let body = r#"
        <text:p>Kept<text:note><text:note-body><text:p>footnote</text:p></text:note-body></text:note></text:p>
        <table:table><table:table-row><table:table-cell><text:p>cell</text:p></table:table-cell></table:table-row></table:table>
        <text:p>After</text:p>
    "#;
    let imported = import_odt(&odt(body)).unwrap();
    let all: Vec<String> = imported.story.iter().map(text).collect();
    assert_eq!(all, vec!["Kept".to_string(), "After".to_string()]);
    let notes = imported.report.notes.join(" ");
    assert!(notes.contains("Tables") && notes.contains("Footnotes"));
}

#[test]
fn something_that_is_not_an_odt_file_is_refused() {
    assert!(matches!(import_odt(b"plain"), Err(OdtError::NotOdt)));
}
