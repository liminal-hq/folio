// Tests for reading a Rich Text file into a story
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_import::{import_rtf, RtfError};

fn text(para: &folio_model::StoryPara) -> String {
    para.inline.iter().map(|r| r.text.as_str()).collect()
}

#[test]
fn paragraphs_emphasis_and_escapes_come_across() {
    let rtf = br"{\rtf1\ansi{\fonttbl{\f0 Times;}}{\colortbl;\red0\green0\blue0;}
\pard First \b bold\b0  and \i slanted\i0 .\par
\pard Caf\'e9 \u8212? dash and \{braces\}.\par
}";
    let imported = import_rtf(rtf).unwrap();
    let s = &imported.story;
    assert_eq!(s.len(), 2);
    assert_eq!(text(&s[0]), "First bold and slanted.");
    assert!(s[0].inline.iter().any(|r| r.bold && r.text == "bold"));
    assert!(s[0].inline.iter().any(|r| r.italic && r.text == "slanted"));
    assert_eq!(text(&s[1]), "Café \u{2014} dash and {braces}.");
    assert!(imported
        .report
        .notes
        .iter()
        .any(|n| n.contains("body text")));
}

#[test]
fn tables_of_fonts_and_pictures_are_left_out() {
    let rtf = br"{\rtf1{\fonttbl{\f0 Times;}}Kept{\pict\wmetafile8 0102}\par After\par}";
    let imported = import_rtf(rtf).unwrap();
    let all: Vec<String> = imported.story.iter().map(text).collect();
    assert_eq!(all, vec!["Kept".to_string(), "After".to_string()]);
    assert!(imported.report.notes.iter().any(|n| n.contains("Pictures")));
}

#[test]
fn something_that_is_not_rtf_is_refused() {
    assert!(matches!(import_rtf(b"plain words"), Err(RtfError::NotRtf)));
}
