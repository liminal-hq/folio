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

#[test]
fn underlined_and_struck_through_runs_come_across() {
    let rtf = br"{\rtf1\ansi\pard A \ul line\ulnone  and \strike gone\strike0 .\par}";
    let s = import_rtf(rtf).unwrap().story;
    assert!(s[0].inline.iter().any(|r| r.underline && r.text == "line"));
    assert!(s[0].inline.iter().any(|r| r.strike && r.text == "gone"));
    assert!(s[0]
        .inline
        .iter()
        .any(|r| !r.underline && !r.strike && r.text == " and "));
}

#[test]
fn sizes_and_expansion_come_across_when_they_are_not_the_bodys() {
    let rtf = br"{\rtf1\ansi\pard\fs22 The body is set in eleven points for a while \fs48\expndtw20 Big\plain\fs22  and back.\par}";
    let s = import_rtf(rtf).unwrap().story;
    let big = s[0].inline.iter().find(|r| r.text == "Big").unwrap();
    assert_eq!(big.size, Some(24.0));
    assert_eq!(big.tracking, Some(42));
    assert!(s[0]
        .inline
        .iter()
        .filter(|r| r.text != "Big")
        .all(|r| r.size.is_none() && r.tracking.is_none()));
}

#[test]
fn a_language_that_is_not_the_bodys_comes_across() {
    let rtf = br"{\rtf1\ansi\pard\lang4105 The body goes on in the usual language for a while, and then \lang3084 bonjour\par}";
    let s = import_rtf(rtf).unwrap().story;
    let bonjour = s[0]
        .inline
        .iter()
        .find(|r| r.text.contains("bonjour"))
        .unwrap();
    assert_eq!(bonjour.lang.as_deref(), Some("fr-CA"));
    assert!(s[0].inline.iter().any(|r| r.lang.is_none()));
}
