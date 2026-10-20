// Tests for reading a Word file into a story
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::io::{Cursor, Write};

use folio_import::{import_docx, DocxError};

fn docx(body: &str, comments: bool) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut out);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        write!(
            zip,
            "<w:document xmlns:w=\"w\"><w:body>{body}</w:body></w:document>"
        )
        .unwrap();
        if comments {
            zip.start_file("word/comments.xml", options).unwrap();
            zip.write_all(b"<w:comments xmlns:w=\"w\"/>").unwrap();
        }
        zip.finish().unwrap();
    }
    out.into_inner()
}

fn text(para: &folio_model::StoryPara) -> String {
    para.inline.iter().map(|r| r.text.as_str()).collect()
}

#[test]
fn headings_quotes_and_emphasis_come_across() {
    let body = r#"
        <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>The kettle</w:t></w:r></w:p>
        <w:p><w:r><w:t xml:space="preserve">She </w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:t>sang</w:t></w:r><w:r><w:t>.</w:t></w:r></w:p>
        <w:p><w:pPr><w:pStyle w:val="Quote"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Boil it.</w:t></w:r></w:p>
        <w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>Later</w:t></w:r></w:p>
    "#;
    let imported = import_docx(&docx(body, false)).unwrap();
    let story = &imported.story;
    assert_eq!(story.len(), 4);
    assert_eq!(story[0].style.as_ref().map(|s| s.0.as_str()), Some("h1"));
    assert_eq!(text(&story[0]), "The kettle");
    assert_eq!(text(&story[1]), "She sang.");
    assert!(story[1].inline.iter().any(|r| r.italic && r.text == "sang"));
    assert_eq!(
        story[2].style.as_ref().map(|s| s.0.as_str()),
        Some("epigraph")
    );
    assert!(story[2].inline[0].bold);
    assert_eq!(story[3].style.as_ref().map(|s| s.0.as_str()), Some("h2"));
    assert_eq!(imported.report.paragraphs, 4);
    assert_eq!(imported.report.headings, 2);
}

#[test]
fn deleted_words_tables_pictures_and_comments_are_left_out_and_said() {
    let body = r#"
        <w:p><w:r><w:t>Kept</w:t></w:r><w:del><w:r><w:delText>gone</w:delText></w:r></w:del><w:ins><w:r><w:t> added</w:t></w:r></w:ins></w:p>
        <w:tbl><w:tr><w:tc><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
        <w:p><w:r><w:drawing><w:t>alt</w:t></w:drawing></w:r><w:r><w:t>After</w:t></w:r></w:p>
    "#;
    let imported = import_docx(&docx(body, true)).unwrap();
    let all: Vec<String> = imported.story.iter().map(text).collect();
    assert_eq!(all, vec!["Kept added".to_string(), "After".to_string()]);
    let notes = imported.report.notes.join(" ");
    for said in ["Comments", "Tables", "Pictures"] {
        assert!(notes.contains(said), "{said} is reported in {notes:?}");
    }
}

#[test]
fn list_items_start_with_a_dash() {
    let body = r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="3"/></w:numPr></w:pPr><w:r><w:t>Tea</w:t></w:r></w:p>"#;
    let imported = import_docx(&docx(body, false)).unwrap();
    assert_eq!(text(&imported.story[0]), "\u{2013} Tea");
}

#[test]
fn something_that_is_not_a_word_file_is_refused() {
    assert!(matches!(
        import_docx(b"plain words"),
        Err(DocxError::NotDocx)
    ));
}
