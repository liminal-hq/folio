// Tests for how a book's sections number its pages, against the cases the canvas also runs
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{page_labels, Page, PageId, Section, Spread, SpreadId, SpreadKind};
use serde_json::Value;

fn spreads_of(case: &Value) -> Vec<Spread> {
    case["spreads"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, s)| Spread {
            id: SpreadId(format!("sp_{i}")),
            kind: serde_json::from_value(s["kind"].clone()).unwrap(),
            pages: (0..s["pages"].as_u64().unwrap())
                .map(|p| Page {
                    id: PageId(format!("pg_{i}_{p}")),
                    master: None,
                    bg: None,
                })
                .collect(),
            frames: Vec::new(),
            guides: Vec::new(),
            master_overrides: Vec::new(),
            bg_span: None,
            section: s
                .get("section")
                .map(|v| serde_json::from_value::<Section>(v.clone()).unwrap()),
        })
        .collect()
}

#[test]
fn every_case_gives_the_labels_and_section_names_the_canvas_gives() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../../apps/folio/src/books/fixtures/labelCases.json"
    ))
    .unwrap();
    for case in cases["cases"].as_array().unwrap() {
        let spreads = spreads_of(case);
        let got = page_labels(&spreads);
        let strings = |key: &str| -> Vec<Vec<String>> {
            case[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| {
                    row.as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap().to_owned())
                        .collect()
                })
                .collect()
        };
        let labels: Vec<Vec<String>> = got
            .iter()
            .map(|row| row.iter().map(|l| l.label.clone()).collect())
            .collect();
        let sections: Vec<Vec<String>> = got
            .iter()
            .map(|row| row.iter().map(|l| l.section.clone()).collect())
            .collect();
        assert_eq!(labels, strings("labels"), "{}", case["name"]);
        assert_eq!(sections, strings("sections"), "{}", case["name"]);
    }
}

#[test]
fn a_cover_is_never_a_section_start_in_the_count() {
    let spreads = vec![Spread {
        id: SpreadId("c".into()),
        kind: SpreadKind::Cover,
        pages: vec![Page {
            id: PageId("p".into()),
            master: None,
            bg: None,
        }],
        frames: Vec::new(),
        guides: Vec::new(),
        master_overrides: Vec::new(),
        bg_span: None,
        section: Some(Section {
            name: "Ignored".into(),
            numbering: folio_model::Numbering::Roman,
            start: Some(9),
        }),
    }];
    assert_eq!(page_labels(&spreads)[0][0].label, "");
}
