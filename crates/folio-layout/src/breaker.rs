// Line breaking: Knuth–Plass optimal breaks, and a greedy first-fit for live typing
//
// Works on the box, glue and penalty items of Knuth and Plass, "Breaking Paragraphs into Lines"
// (1981), and knows nothing about fonts or text: widths are plain numbers in points.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// A penalty that forbids a break, and one that forces it.
pub const INFINITE: f64 = 10_000.0;

/// What a paragraph is made of, in order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Item {
    /// Something that cannot be split: a word or a piece of one.
    Box { width: f64 },
    /// Space that may stretch and shrink, and where a line may end (after a box).
    Glue {
        width: f64,
        stretch: f64,
        shrink: f64,
    },
    /// A possible break that costs `penalty`. `width` is added only when the line ends here (the
    /// hyphen). `flagged` marks a hyphen, so two in a row can be discouraged.
    Penalty {
        width: f64,
        penalty: f64,
        flagged: bool,
    },
}

impl Item {
    pub fn glue(width: f64, stretch: f64, shrink: f64) -> Self {
        Item::Glue {
            width,
            stretch,
            shrink,
        }
    }

    /// The end of a paragraph, or of a line that must not be joined to the next: glue that
    /// stretches without limit, then a forced break.
    pub fn forced_break() -> [Item; 2] {
        [
            Item::glue(0.0, 1.0e9, 0.0),
            Item::Penalty {
                width: 0.0,
                penalty: -INFINITE,
                flagged: false,
            },
        ]
    }
}

/// How costly different lines are. The defaults follow TeX.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    /// The loosest a line may be, as a stretch ratio. Tried first; raised when no breaks fit.
    pub tolerance: f64,
    pub line_penalty: f64,
    /// Added when two lines in a row end in a hyphen.
    pub flagged_demerit: f64,
    /// Added when neighbouring lines differ by more than one fitness class (tight next to loose).
    pub fitness_demerit: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            tolerance: 1.0,
            line_penalty: 10.0,
            flagged_demerit: 3000.0,
            fitness_demerit: 100.0,
        }
    }
}

/// One finished line: where it ends, and how far its glue was stretched (negative: shrunk).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Break {
    /// Index of the glue or penalty item the line ends at.
    pub at: usize,
    pub ratio: f64,
}

#[derive(Clone, Copy)]
struct Totals {
    width: f64,
    stretch: f64,
    shrink: f64,
}

struct Node {
    at: usize,
    line: usize,
    fitness: usize,
    totals: Totals,
    demerits: f64,
    ratio: f64,
    previous: Option<usize>,
}

fn fitness_class(ratio: f64) -> usize {
    if ratio < -0.5 {
        0
    } else if ratio <= 0.5 {
        1
    } else if ratio <= 1.0 {
        2
    } else {
        3
    }
}

/// The line's stretch ratio, `f64::NEG_INFINITY` when it is too long to shrink to fit and
/// `f64::INFINITY` when it is short and cannot stretch.
fn ratio(width: f64, stretch: f64, shrink: f64, target: f64) -> f64 {
    if width < target {
        if stretch > 0.0 {
            (target - width) / stretch
        } else {
            f64::INFINITY
        }
    } else if width > target {
        if shrink > 0.0 {
            (target - width) / shrink
        } else {
            f64::NEG_INFINITY
        }
    } else {
        0.0
    }
}

/// The best breaks for `items`, which must end with a forced break. `width_of(n)` is the width of
/// line `n` (from 0), so the first line may be indented. `None` when no breaks keep every line
/// within `tolerance`.
fn optimal(
    items: &[Item],
    width_of: &dyn Fn(usize) -> f64,
    tolerance: f64,
    params: &Params,
) -> Option<Vec<Break>> {
    let zero = Totals {
        width: 0.0,
        stretch: 0.0,
        shrink: 0.0,
    };
    let mut nodes = vec![Node {
        at: 0,
        line: 0,
        fitness: 1,
        totals: zero,
        demerits: 0.0,
        ratio: 0.0,
        previous: None,
    }];
    let mut active: Vec<usize> = vec![0];
    let mut sum = zero;

    let consider = |i: usize, sum: Totals, nodes: &mut Vec<Node>, active: &mut Vec<usize>| {
        let (penalty, pwidth, flagged) = match items[i] {
            Item::Penalty {
                width,
                penalty,
                flagged,
            } => (penalty, width, flagged),
            _ => (0.0, 0.0, false),
        };
        let mut best: [Option<(f64, usize, f64)>; 4] = [None; 4];
        let mut at = 0;
        while at < active.len() {
            let a = active[at];
            let node = &nodes[a];
            let r = ratio(
                sum.width - node.totals.width + pwidth,
                sum.stretch - node.totals.stretch,
                sum.shrink - node.totals.shrink,
                width_of(node.line),
            );
            if r < -1.0 || penalty <= -INFINITE {
                active.remove(at);
            } else {
                at += 1;
            }
            if (-1.0..=tolerance).contains(&r) {
                let badness = 100.0 * r.abs().powi(3);
                let mut demerits = if penalty >= 0.0 {
                    (params.line_penalty + badness).powi(2) + penalty.powi(2)
                } else if penalty > -INFINITE {
                    (params.line_penalty + badness).powi(2) - penalty.powi(2)
                } else {
                    (params.line_penalty + badness).powi(2)
                };
                let previous_flagged =
                    matches!(items[node.at], Item::Penalty { flagged: true, .. });
                if flagged && previous_flagged {
                    demerits += params.flagged_demerit;
                }
                let class = fitness_class(r);
                if class.abs_diff(node.fitness) > 1 {
                    demerits += params.fitness_demerit;
                }
                demerits += node.demerits;
                if best[class].is_none_or(|(d, _, _)| demerits < d) {
                    best[class] = Some((demerits, a, r));
                }
            }
        }
        // Totals from the break on, up to the next box: the glue after a break disappears.
        let mut after = sum;
        for (j, item) in items.iter().enumerate().skip(i) {
            match *item {
                Item::Glue {
                    width,
                    stretch,
                    shrink,
                } => {
                    after.width += width;
                    after.stretch += stretch;
                    after.shrink += shrink;
                }
                Item::Box { .. } => break,
                Item::Penalty { penalty, .. } if penalty <= -INFINITE && j > i => break,
                Item::Penalty { .. } => {}
            }
        }
        for (class, found) in best.iter().enumerate() {
            if let Some((demerits, from, r)) = *found {
                let line = nodes[from].line + 1;
                nodes.push(Node {
                    at: i,
                    line,
                    fitness: class,
                    totals: after,
                    demerits,
                    ratio: r,
                    previous: Some(from),
                });
                active.push(nodes.len() - 1);
            }
        }
    };

    for (i, item) in items.iter().enumerate() {
        match *item {
            Item::Box { width } => sum.width += width,
            Item::Glue {
                width,
                stretch,
                shrink,
            } => {
                if i > 0 && matches!(items[i - 1], Item::Box { .. }) {
                    consider(i, sum, &mut nodes, &mut active);
                }
                sum.width += width;
                sum.stretch += stretch;
                sum.shrink += shrink;
            }
            Item::Penalty { penalty, .. } => {
                if penalty < INFINITE {
                    consider(i, sum, &mut nodes, &mut active);
                }
            }
        }
        if active.is_empty() {
            return None;
        }
    }

    let end = *active
        .iter()
        .min_by(|&&a, &&b| nodes[a].demerits.total_cmp(&nodes[b].demerits))?;
    let mut breaks = Vec::new();
    let mut at = Some(end);
    while let Some(n) = at {
        let node = &nodes[n];
        if node.previous.is_some() {
            breaks.push(Break {
                at: node.at,
                ratio: node.ratio,
            });
        }
        at = node.previous;
    }
    breaks.reverse();
    Some(breaks)
}

/// How a paragraph ended up broken.
#[derive(Debug, Clone, PartialEq)]
pub struct Broken {
    pub breaks: Vec<Break>,
    /// The tolerance that worked, or `None` when only first-fit could break it (some line is
    /// overfull, or looser than any tolerance allows).
    pub tolerance: Option<f64>,
}

/// The tolerances tried in turn before giving up on an optimal fit.
const TOLERANCES: [f64; 5] = [1.0, 2.0, 4.0, 10.0, 50.0];

/// Breaks `items` into lines of the widths `width_of` gives, the best way: Knuth–Plass at the
/// tightest tolerance that works, then first-fit if none does.
pub fn break_lines(items: &[Item], width_of: &dyn Fn(usize) -> f64, params: &Params) -> Broken {
    for tolerance in TOLERANCES.into_iter().filter(|t| *t >= params.tolerance) {
        if let Some(breaks) = optimal(items, width_of, tolerance, params) {
            return Broken {
                breaks,
                tolerance: Some(tolerance),
            };
        }
    }
    Broken {
        breaks: first_fit(items, width_of),
        tolerance: None,
    }
}

/// First-fit: each line takes as much as fits. Fast, so it serves while the person is typing.
/// A word wider than the line stands alone on one, overfull.
pub fn first_fit(items: &[Item], width_of: &dyn Fn(usize) -> f64) -> Vec<Break> {
    let mut breaks = Vec::new();
    let mut line = 0;
    let mut width = 0.0;
    let mut stretch = 0.0;
    let mut shrink = 0.0;
    // The last place the line could have ended, with the width the line would have had there.
    let mut candidate: Option<(usize, f64, f64, f64)> = None;
    let mut i = 0;
    let mut start = 0;
    while i < items.len() {
        // Only a box that does not fit pushes a line over: the finishing glue never does.
        let mut overflow = false;
        match items[i] {
            Item::Box { width: w } => {
                width += w;
                overflow = true;
            }
            Item::Glue {
                width: w,
                stretch: s,
                shrink: k,
            } => {
                if i > 0 && matches!(items[i - 1], Item::Box { .. }) {
                    candidate = Some((i, width, stretch, shrink));
                }
                width += w;
                stretch += s;
                shrink += k;
            }
            Item::Penalty {
                width: w, penalty, ..
            } => {
                if penalty <= -INFINITE {
                    breaks.push(Break {
                        at: i,
                        ratio: ratio(width, stretch, shrink, width_of(line)).clamp(-1.0, 0.0),
                    });
                    line += 1;
                    width = 0.0;
                    stretch = 0.0;
                    shrink = 0.0;
                    candidate = None;
                    start = i + 1;
                } else if penalty < INFINITE {
                    candidate = Some((i, width + w, stretch, shrink));
                }
            }
        }
        if overflow
            && width - shrink > width_of(line) + 1e-9
            && candidate.is_some_and(|(c, ..)| c > start)
        {
            let (at, w, s, k) = candidate.take().expect("checked above");
            breaks.push(Break {
                at,
                ratio: ratio(w, s, k, width_of(line)),
            });
            line += 1;
            // Resume after the break: skip the glue that ends the line.
            width = 0.0;
            stretch = 0.0;
            shrink = 0.0;
            start = at + 1;
            i = at;
            while let Some(Item::Glue { .. }) = items.get(i + 1) {
                i += 1;
                start = i + 1;
            }
            i += 1;
            continue;
        }
        i += 1;
    }
    breaks
}
