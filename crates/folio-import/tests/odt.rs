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
             <style:style style:name=\"T3\" style:family=\"text\"><style:text-properties style:text-underline-style=\"solid\"/></style:style>\
             <style:style style:name=\"T4\" style:family=\"text\"><style:text-properties style:text-line-through-style=\"solid\" style:text-underline-style=\"none\"/></style:style>\
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

#[test]
fn the_styles_found_are_listed_and_a_choice_changes_what_a_style_becomes() {
    use folio_import::{import_odt_with, StyleMap};
    let body = r#"
        <text:h text:outline-level="2">Part</text:h>
        <text:p text:style-name="Caption">A caption</text:p>
        <text:p>Plain</text:p>
    "#;
    let bytes = odt(body);
    let first = import_odt(&bytes).unwrap();
    let names: Vec<&str> = first
        .report
        .styles
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    assert_eq!(names, vec!["Caption", "Heading 2", "Standard"]);
    let mut map = StyleMap::new();
    map.insert("Heading 2".into(), "h1".into());
    map.insert("Caption".into(), "epigraph".into());
    let second = import_odt_with(&bytes, &map).unwrap();
    let styles: Vec<Option<&str>> = second
        .story
        .iter()
        .map(|p| p.style.as_ref().map(|s| s.0.as_str()))
        .collect();
    assert_eq!(styles, vec![Some("h1"), Some("epigraph"), None]);
    assert_eq!(second.report.headings, 1);
}

#[test]
fn underlined_and_struck_through_spans_come_across() {
    let body = r#"<text:p>An <text:span text:style-name="T3">underlined</text:span> and a <text:span text:style-name="T4">struck</text:span> word.</text:p>"#;
    let s = import_odt(&odt(body)).unwrap().story;
    assert!(s[0]
        .inline
        .iter()
        .any(|r| r.underline && !r.strike && r.text == "underlined"));
    assert!(s[0]
        .inline
        .iter()
        .any(|r| r.strike && !r.underline && r.text == "struck"));
    assert!(s[0].inline.iter().any(|r| !r.underline && !r.strike));
}

#[test]
fn a_spans_size_and_letter_spacing_come_across() {
    let mut out = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut out);
        zip.start_file("content.xml", zip::write::SimpleFileOptions::default())
            .unwrap();
        write!(
            zip,
            "<office:document-content xmlns:office=\"o\" xmlns:text=\"t\" xmlns:style=\"s\" xmlns:fo=\"f\">\
             <office:automatic-styles>\
             <style:style style:name=\"T1\" style:family=\"text\"><style:text-properties fo:font-size=\"24pt\" fo:letter-spacing=\"1.2pt\"/></style:style>\
             </office:automatic-styles><office:body><office:text>\
             <text:p>Plain words and <text:span text:style-name=\"T1\">Large</text:span> ones.</text:p>\
             </office:text></office:body></office:document-content>"
        )
        .unwrap();
        zip.finish().unwrap();
    }
    let story = import_odt(&out.into_inner()).unwrap().story;
    let large = story[0].inline.iter().find(|r| r.text == "Large").unwrap();
    assert_eq!(large.size, Some(24.0));
    assert_eq!(large.tracking, Some(50));
    assert!(story[0].inline.iter().any(|r| r.size.is_none()));
}

#[test]
fn a_spans_language_comes_across_and_the_bodys_is_left_to_the_book() {
    let mut out = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut out);
        zip.start_file("content.xml", zip::write::SimpleFileOptions::default())
            .unwrap();
        write!(
            zip,
            "<office:document-content xmlns:office=\"o\" xmlns:text=\"t\" xmlns:style=\"s\" xmlns:fo=\"f\">\
             <office:automatic-styles>\
             <style:style style:name=\"E\" style:family=\"text\"><style:text-properties fo:language=\"en\" fo:country=\"CA\"/></style:style>\
             <style:style style:name=\"F\" style:family=\"text\"><style:text-properties fo:language=\"fr\" fo:country=\"CA\"/></style:style>\
             <style:style style:name=\"N\" style:family=\"text\"><style:text-properties fo:language=\"zxx\"/></style:style>\
             </office:automatic-styles><office:body><office:text>\
             <text:p><text:span text:style-name=\"E\">The body goes on in the usual language for a while, and then </text:span><text:span text:style-name=\"F\">bonjour</text:span><text:span text:style-name=\"N\"> code</text:span></text:p>\
             </office:text></office:body></office:document-content>"
        )
        .unwrap();
        zip.finish().unwrap();
    }
    let runs = import_odt(&out.into_inner()).unwrap().story[0]
        .inline
        .clone();
    let bonjour = runs.iter().find(|r| r.text == "bonjour").unwrap();
    assert_eq!(bonjour.lang.as_deref(), Some("fr-CA"));
    assert!(runs
        .iter()
        .filter(|r| r.text != "bonjour")
        .all(|r| r.lang.is_none()));
}
