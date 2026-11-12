// Tests for reading pasted HTML into a story: snippets as Chrome, Google Docs, Word and others write them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_import::{import_html, HtmlError, MAX_HTML_BYTES};
use folio_model::{Run, StoryPara};

/// Typefaces this "computer" has.
fn installed(name: &str) -> bool {
    matches!(name, "Calibri" | "Literata" | "Georgia")
}

fn read(html: &str) -> Vec<StoryPara> {
    import_html(html, &installed).unwrap().story
}

fn text(para: &StoryPara) -> String {
    para.inline.iter().map(|r| r.text.as_str()).collect()
}

fn run<'a>(para: &'a StoryPara, word: &str) -> &'a Run {
    para.inline
        .iter()
        .find(|r| r.text.contains(word))
        .unwrap_or_else(|| panic!("no run holds {word:?} in {:?}", para.inline))
}

#[test]
fn a_browser_copy_with_a_computed_style_on_every_span_keeps_only_what_differs() {
    let html = r#"<meta charset='utf-8'><span style="color: rgb(34, 34, 34); font-family: Arial, sans-serif; font-size: 16px; font-style: normal; font-weight: 400; text-decoration: none;">The quick </span><b style="color: rgb(34, 34, 34); font-family: Arial, sans-serif; font-size: 16px; font-weight: 700;">brown</b><span style="color: rgb(34, 34, 34); font-family: Arial, sans-serif; font-size: 16px;"> fox, </span><a href="https://example.org/fox?a=1&amp;b=2" style="color: rgb(0, 0, 238); font-size: 16px; text-decoration: underline;">jumps</a><span style="color: rgb(34, 34, 34); font-size: 16px;"> </span><em style="color: rgb(200, 30, 30); font-size: 16px;">over</em>"#;
    let story = read(html);
    assert_eq!(story.len(), 1);
    let para = &story[0];
    assert_eq!(text(para), "The quick brown fox, jumps over");
    assert!(run(para, "brown").bold);
    assert!(!run(para, "quick").bold);
    // The ink of the page it came from is not kept, nor its size, nor a face that is not installed.
    assert!(para
        .inline
        .iter()
        .all(|r| r.size.is_none() && r.font.is_none()));
    assert!(run(para, "quick").colour.is_none());
    // A link keeps its address and none of the page's blue or underline.
    let link = run(para, "jumps");
    assert_eq!(
        link.href.as_deref(),
        Some("https://example.org/fox?a=1&b=2")
    );
    assert!(link.colour.is_none() && !link.underline);
    // A colour that is not the page's ink stays.
    let over = run(para, "over");
    assert!(over.italic);
    assert_eq!(over.colour.as_ref().map(|c| c.0.as_str()), Some("#c81e1e"));
}

#[test]
fn google_docs_wraps_everything_in_a_bold_that_is_not_bold() {
    let html = r#"<meta charset='utf-8'><meta charset="utf-8"><b style="font-weight:normal;" id="docs-internal-guid-1a2b"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">Plain, </span><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;font-weight:700;white-space:pre-wrap;">bold</span><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;font-style:italic;white-space:pre-wrap;"> and italic</span></p><p dir="ltr" style="line-height:1.38;"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;text-decoration:underline;-webkit-text-decoration-skip:none;white-space:pre-wrap;">Underlined</span><span style="font-size:11pt;color:#ff0000;white-space:pre-wrap;"> red </span><span style="font-size:11pt;text-decoration:line-through;white-space:pre-wrap;">struck</span></p><br /></b>"#;
    let story = read(html);
    assert_eq!(story.len(), 2);
    assert_eq!(text(&story[0]), "Plain, bold and italic");
    assert!(!run(&story[0], "Plain").bold);
    assert!(run(&story[0], "bold").bold);
    assert!(run(&story[0], "italic").italic);
    assert_eq!(text(&story[1]), "Underlined red struck");
    assert!(run(&story[1], "Underlined").underline);
    assert_eq!(
        run(&story[1], "red").colour.as_ref().map(|c| c.0.as_str()),
        Some("#ff0000")
    );
    assert!(run(&story[1], "struck").strike);
}

#[test]
fn a_word_paste_drops_its_comments_styles_and_office_markup_and_keeps_a_larger_heading() {
    let html = r#"<html xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:w="urn:schemas-microsoft-com:office:word"><head><meta http-equiv=Content-Type content="text/html; charset=utf-8"><!--[if gte mso 9]><xml><o:OfficeDocumentSettings><o:AllowPNG/></o:OfficeDocumentSettings></xml><![endif]--><style><!-- p.MsoNormal, li.MsoNormal { margin:0in; font-size:11.0pt; } --></style></head><body lang=EN-US style='tab-interval:.5in'><!--StartFragment--><p class=MsoNormal><b style='mso-bidi-font-weight:normal'><span style='font-size:14.0pt;font-family:"Calibri",sans-serif'>Chapter One<o:p></o:p></span></b></p>

<p class=MsoNormal><span style='font-family:"Calibri",sans-serif'>It was <i>cold</i> and <u>wet</u>,<span style='mso-spacerun:yes'>  </span>said <s>Ann</s> &#8212; and &ldquo;quiet&rdquo;.<o:p></o:p></span></p>

<p class=MsoNormal><o:p>&nbsp;</o:p></p><!--EndFragment--></body></html>"#;
    let story = read(html);
    assert_eq!(story.len(), 2, "{story:?}");
    assert_eq!(text(&story[0]), "Chapter One");
    assert!(story[0].inline[0].bold);
    // The heading is larger than the rest, which is the body size and is left to the frame.
    assert_eq!(story[0].inline[0].size, Some(14.0));
    assert_eq!(
        text(&story[1]),
        "It was cold and wet, said Ann \u{2014} and \u{201c}quiet\u{201d}."
    );
    assert!(story[1].inline.iter().all(|r| r.size.is_none()));
    assert!(run(&story[1], "cold").italic && run(&story[1], "wet").underline);
    assert!(run(&story[1], "Ann").strike);
    // Calibri is installed, and as every paragraph is in it, it is the frame's to choose.
    assert!(story
        .iter()
        .flat_map(|p| &p.inline)
        .all(|r| r.font.is_none()));
    let all: String = story.iter().map(text).collect();
    assert!(!all.contains("MsoNormal") && !all.contains("OfficeDocumentSettings"));
}

#[test]
fn a_typeface_is_kept_only_if_installed_and_only_where_it_differs() {
    let html = r#"<p style="font-family: Georgia, serif">In Georgia, <span style="font-family: 'Comic Neue', 'Literata', cursive">then Literata</span>, <span style="font-family: 'Comic Neue', cursive">then neither</span>.</p>"#;
    let story = read(html);
    let para = &story[0];
    // Georgia is most of the words and is the frame's to choose; Literata differs and stays.
    assert_eq!(run(para, "Literata").font.as_deref(), Some("Literata"));
    assert!(run(para, "In Georgia").font.is_none());
    assert!(run(para, "neither").font.is_none());
    // Alone, a face that is installed is the paste's own only when something else is not.
    let alone = read(r#"<p><span style="font-family: Literata">only</span> plain</p>"#);
    assert_eq!(run(&alone[0], "only").font.as_deref(), Some("Literata"));
}

#[test]
fn lists_tables_breaks_and_pre_become_paragraphs() {
    let html = "<h2>Shopping</h2><ul><li>eggs</li><li>milk <b>2%</b></li></ul><ol><li>first</li><li>second<ul><li>nested</li></ul></li></ol>\
                <table><tr><th>Name</th><th>Qty</th></tr><tr><td>eggs</td><td>12</td></tr></table>\
                <p>one<br>two<br><br>three</p><pre>line a\n  line b</pre>";
    let made = import_html(html, &installed).unwrap();
    let lines: Vec<String> = made.story.iter().map(text).collect();
    assert_eq!(
        lines,
        vec![
            "Shopping",
            "\u{2013} eggs",
            "\u{2013} milk 2%",
            "1. first",
            "2. second",
            "\u{2013} nested",
            "Name Qty",
            "eggs 12",
            "one",
            "two",
            "three",
            "line a",
            "  line b",
        ]
    );
    assert!(made
        .report
        .notes
        .iter()
        .any(|n| n.starts_with("Tables came in")));
    assert_eq!(made.report.paragraphs, 13);
}

#[test]
fn scripts_styles_hidden_things_pictures_and_unknown_elements_never_come_in() {
    let html = r#"<div>Hello <script>alert('<b>x</b>')</script><style>p { color: red }</style><span style="display:none">secret</span><span hidden>also secret</span><img src="x.png" onerror="alert(1)" alt="picture words"><iframe src="https://evil.example">frame words</iframe><svg><text>svg words</text></svg><custom-element onclick="x()">kept words</custom-element><input value="field"><a href="javascript:alert(1)">bad link</a> <a href="chapter2.html">relative</a> <a href="mailto:me@example.org">mail</a> world</div><title>Title words</title><textarea>area words</textarea>"#;
    let made = import_html(html, &installed).unwrap();
    let all: String = made.story.iter().map(text).collect::<Vec<_>>().join("|");
    assert_eq!(
        all,
        "Hello kept wordsbad link relative mail world|area words"
    );
    for forbidden in [
        "alert",
        "secret",
        "picture words",
        "svg words",
        "frame words",
        "<",
        "onerror",
        "Title",
    ] {
        assert!(!all.contains(forbidden), "{forbidden}");
    }
    let para = &made.story[0];
    assert!(run(para, "bad link").href.is_none());
    assert!(run(para, "relative").href.is_none());
    assert_eq!(
        run(para, "mail").href.as_deref(),
        Some("mailto:me@example.org")
    );
    assert!(made
        .report
        .notes
        .iter()
        .any(|n| n.starts_with("Pictures were left out")));
}

#[test]
fn entities_whitespace_and_a_tag_that_never_ends_are_read_the_way_a_browser_would() {
    let story = read("<p>  Tom &amp; Jerry &lt;3&nbsp;&#x41;&#66;&bogus; \n\t  spaced   out &copy;</p><p>5 < 6 and 7 > 3</p><p>unfinished <b");
    assert_eq!(
        text(&story[0]),
        "Tom & Jerry <3\u{a0}AB&bogus; spaced out \u{a9}"
    );
    assert_eq!(text(&story[1]), "5 < 6 and 7 > 3");
    // The words before a tag that never ends are kept; the tag is not.
    assert_eq!(story.len(), 3);
    assert_eq!(text(&story[2]), "unfinished");
    // Plain text with no markup is a paragraph.
    assert_eq!(text(&read("just words")[0]), "just words");
    assert!(read("").is_empty());
    assert!(read("<div> \n <p>&nbsp;</p></div>").is_empty());
}

#[test]
fn the_fragment_between_the_markers_is_what_is_read() {
    let html = "Version:0.9\r\nStartHTML:0000000105\r\n<html><body>\r\n<!--StartFragment--><p>copied <b>words</b></p><!--EndFragment-->\r\n</body>stray words</html>";
    let story = read(html);
    assert_eq!(story.len(), 1);
    assert_eq!(text(&story[0]), "copied words");
}

#[test]
fn colours_that_are_the_pages_ink_or_paper_are_dropped_and_others_kept() {
    let colour = |css: &str| {
        let made = read(&format!("<p>x <span style=\"color:{css}\">y</span> z</p>"));
        made[0]
            .inline
            .iter()
            .find(|r| r.text.contains('y'))
            .and_then(|r| r.colour.clone())
            .map(|c| c.0)
    };
    for ink in [
        "#000",
        "black",
        "#222222",
        "rgb(34, 34, 34)",
        "#fff",
        "white",
        "rgb(255,255,255)",
        "transparent",
        "windowtext",
        "rgba(200,0,0,0.1)",
        "inherit",
    ] {
        assert_eq!(colour(ink), None, "{ink}");
    }
    assert_eq!(colour("#c00").as_deref(), Some("#cc0000"));
    assert_eq!(colour("rgb(10% 50% 90%)").as_deref(), Some("#1a7fe5"));
    assert_eq!(colour("rgb(0, 100, 200)").as_deref(), Some("#0064c8"));
    assert_eq!(colour("Teal").as_deref(), Some("#008080"));
    assert_eq!(colour("not-a-colour"), None);
}

#[test]
fn old_font_tags_set_a_face_a_colour_and_a_size() {
    let story = read(
        r##"<p>plain plain plain <font face="Georgia" color="#336699" size="5">big blue</font></p>"##,
    );
    let big = run(&story[0], "big");
    assert_eq!(big.size, Some(18.0));
    assert_eq!(big.colour.as_ref().map(|c| c.0.as_str()), Some("#336699"));
    assert_eq!(big.font.as_deref(), Some("Georgia"));
}

#[test]
fn a_paste_over_the_limit_is_refused_and_a_deep_one_is_read() {
    let big = format!("<p>{}</p>", "word ".repeat(MAX_HTML_BYTES / 4));
    assert_eq!(
        import_html(&big, &installed).unwrap_err(),
        HtmlError::TooLarge
    );
    let deep = format!(
        "{}deep{}",
        "<div><b>".repeat(5000),
        "</b></div>".repeat(5000)
    );
    let story = read(&deep);
    assert_eq!(text(&story[0]), "deep");
    assert!(story[0].inline[0].bold);
    let many = "<p>x</p>".repeat(50_000);
    assert_eq!(read(&many).len(), 50_000);
}
