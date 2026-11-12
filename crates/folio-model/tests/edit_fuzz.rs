//! Random edits of paragraphs with different looks, checked against a plain model of the words
//!
//! (c) Copyright 2026 Liminal HQ, Scott Morris
//! SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{
    content_of, paragraphs_of, replace_range, ParaLook, Run, StoryPara, TextInsert, TextPos,
};
use proptest::prelude::*;

/// A paragraph as the plain model keeps it: its words and the space before that marks its look.
type Plain = (String, Option<f64>);

fn look(space: Option<f64>) -> Option<ParaLook> {
    space.map(|s| ParaLook {
        space_before: Some(s),
        ..ParaLook::default()
    })
}

fn story(model: &[Plain]) -> Vec<StoryPara> {
    model
        .iter()
        .map(|(text, space)| StoryPara {
            style: None,
            inline: vec![Run {
                text: text.clone(),
                ..Run::default()
            }],
            look: look(*space),
        })
        .collect()
}

fn read(paras: &[StoryPara]) -> Vec<Plain> {
    paras
        .iter()
        .map(|p| {
            (
                p.inline.iter().map(|r| r.text.as_str()).collect(),
                p.look.as_ref().and_then(|l| l.space_before),
            )
        })
        .collect()
}

/// What `replace_range` should make of the model: the words replaced, a paragraph for each line
/// break typed, and the look of the first paragraph kept — of the last when the first is blank.
fn expected(model: &[Plain], from: (usize, usize), to: (usize, usize), text: &str) -> Vec<Plain> {
    let chars = |s: &str| s.chars().collect::<Vec<_>>();
    let head: String = chars(&model[from.0].0)[..from.1].iter().collect();
    let tail: String = chars(&model[to.0].0)[to.1..].iter().collect();
    let blank_first = to.0 > from.0 && model[from.0].0.is_empty();
    let kept = model[if blank_first { to.0 } else { from.0 }].1;
    let joined = format!("{head}{text}{tail}");
    let made: Vec<Plain> = joined.split('\n').map(|t| (t.to_owned(), kept)).collect();
    let mut out = model[..from.0].to_vec();
    out.extend(made);
    out.extend_from_slice(&model[to.0 + 1..]);
    out
}

/// An edit: where it starts and ends (as fractions of the words, so they are always in range), and
/// what is typed: nothing (a Backspace or Delete), a line break (Enter), or words.
#[derive(Clone, Debug)]
struct Edit {
    from: (usize, usize),
    span: usize,
    text: String,
}

fn edit() -> impl Strategy<Value = Edit> {
    (
        (0usize..64, 0usize..64),
        prop_oneof![Just(0usize), Just(1), 0usize..12],
        prop_oneof![
            Just(String::new()),
            Just("\n".to_owned()),
            "[ab\"]{1,3}",
            "[a\n]{1,4}",
        ],
    )
        .prop_map(|(from, span, text)| Edit { from, span, text })
}

/// Places in the model as (paragraph, character), a line break counting as one place.
fn places(model: &[Plain]) -> Vec<(usize, usize)> {
    model
        .iter()
        .enumerate()
        .flat_map(|(p, (t, _))| (0..=t.chars().count()).map(move |o| (p, o)))
        .collect()
}

fn paragraphs() -> impl Strategy<Value = Vec<Plain>> {
    prop::collection::vec(
        (
            prop_oneof![Just(String::new()), "[a-c ]{1,6}"],
            prop_oneof![Just(None), (1u8..30).prop_map(|n| Some(f64::from(n)))],
        ),
        1..6,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn edits_keep_the_words_the_paragraphs_and_their_looks(
        start in paragraphs(),
        edits in prop::collection::vec(edit(), 1..12),
    ) {
        let mut model = start;
        let mut paras = story(&model);
        for e in edits {
            let all = places(&model);
            let a = all[e.from.0 % all.len()];
            let b = all[((e.from.0 % all.len()) + e.span).min(all.len() - 1)];
            let lines_before = model.len();
            let removed = b.0 - a.0;
            let added = e.text.matches('\n').count();
            let want = expected(&model, a, b, &e.text);
            let caret = replace_range(
                &mut paras,
                TextPos { para: a.0 as u32, offset: a.1 as u32 },
                TextPos { para: b.0 as u32, offset: b.1 as u32 },
                &TextInsert::Text { text: e.text.clone() },
                None,
            )
            .expect("a place in the words");
            let got = read(&paras);
            prop_assert_eq!(&got, &want, "edit {:?} from {:?} to {:?}", e, a, b);
            // A paragraph for each line break typed, one less for each taken away.
            prop_assert_eq!(got.len(), lines_before + added - removed);
            // The caret is in the words, after what was typed.
            prop_assert!((caret.para as usize) < got.len());
            prop_assert!(caret.offset as usize <= got[caret.para as usize].0.chars().count());
            model = want;
            // A frame keeps its words as runs with line breaks and its looks beside them: written
            // that way and read back, every paragraph keeps its words and its look, blank ones too.
            paras = paragraphs_of(Some(&content_of(&paras)));
            prop_assert_eq!(read(&paras), model.clone());
        }
    }
}

#[test]
fn backspace_in_a_blank_paragraph_leaves_its_neighbours_as_they_were() {
    let model: Vec<Plain> = vec![
        ("\"I can't wait,\" said Emma.".into(), Some(8.0)),
        (String::new(), None),
        ("Grandma said, \"That's good.\"".into(), Some(12.0)),
    ];
    // Backspace at the start of the blank paragraph joins it to the one before.
    let mut paras = story(&model);
    let end = model[0].0.chars().count() as u32;
    replace_range(
        &mut paras,
        TextPos {
            para: 0,
            offset: end,
        },
        TextPos { para: 1, offset: 0 },
        &TextInsert::Text {
            text: String::new(),
        },
        None,
    )
    .unwrap();
    assert_eq!(read(&paras), vec![model[0].clone(), model[2].clone()]);

    // Delete in the blank paragraph joins the next to it: the next keeps its own look.
    let mut paras = story(&model);
    replace_range(
        &mut paras,
        TextPos { para: 1, offset: 0 },
        TextPos { para: 2, offset: 0 },
        &TextInsert::Text {
            text: String::new(),
        },
        None,
    )
    .unwrap();
    assert_eq!(read(&paras), vec![model[0].clone(), model[2].clone()]);
}
