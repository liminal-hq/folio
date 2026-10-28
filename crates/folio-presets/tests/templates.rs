// Tests for the spread templates: every one builds for every kind's size, inside the spread
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_presets::templates::{build, list, styles_named, Metrics, GROUPS};
use folio_presets::{kind_infos, preset_styles};

fn counter() -> impl FnMut(&str) -> String {
    let mut n = 0;
    move |prefix| {
        n += 1;
        format!("{prefix}_{n}")
    }
}

#[test]
fn every_template_is_in_a_known_group_with_a_name_and_a_unique_id() {
    let all = list();
    assert!(all.len() >= 20, "{} templates", all.len());
    let mut ids: Vec<&str> = all.iter().map(|t| t.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), all.len(), "ids are unique");
    for template in &all {
        assert!(GROUPS.contains(&template.group.as_str()), "{}", template.id);
        assert!(!template.name.is_empty());
    }
    // Every group has something in it.
    for group in GROUPS {
        assert!(all.iter().any(|t| t.group == group), "{group}");
    }
}

#[test]
fn every_template_names_only_styles_the_presets_have() {
    let styles = preset_styles();
    for name in styles_named() {
        assert!(
            styles.styles.contains_key(name),
            "a template names the style `{name}`"
        );
    }
}

#[test]
fn every_template_fits_the_spread_at_every_size_a_kind_offers() {
    for info in kind_infos() {
        for trim in &info.trims {
            let (w, h) = (trim.width.to_in(), trim.height.to_in());
            let k = Metrics {
                w,
                h,
                m: 0.5_f64.min(w / 8.0),
                b: 0.125,
            };
            for template in list() {
                let mut ids = counter();
                let frames = build(&template.id, &k, &mut ids).expect("it builds");
                assert!(!frames.is_empty(), "{}", template.id);
                for frame in &frames {
                    let r = frame.base().rect;
                    let (x, y, fw, fh) = (r.x.to_in(), r.y.to_in(), r.w.to_in(), r.h.to_in());
                    let slack = 1e-6;
                    assert!(
                        x >= -k.b - slack && y >= -k.b - slack,
                        "{} at {w}×{h}: a frame starts off the page ({x}, {y})",
                        template.id
                    );
                    assert!(
                        x + fw <= 2.0 * w + k.b + slack && y + fh <= h + k.b + slack,
                        "{} at {w}×{h}: a frame ends off the page",
                        template.id
                    );
                    assert!(fw > 0.0 && fh > 0.0, "{} has an empty frame", template.id);
                }
            }
        }
    }
}

#[test]
fn a_template_that_does_not_exist_builds_nothing() {
    let k = Metrics {
        w: 8.5,
        h: 8.5,
        m: 0.5,
        b: 0.125,
    };
    assert!(build("no-such", &k, &mut counter()).is_none());
}
