// The session: the book, its revision, and the undo and redo stacks
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::{Duration, Instant};

use folio_model::{Book, EditCaret, Intent};

use crate::apply::{plan, run, MergeKey};
use crate::delta::Delta;
use crate::error::SessionError;
use crate::patch::{Patch, PatchOp};

/// How many steps are kept to undo.
pub const UNDO_DEPTH: usize = 200;

/// Repeats of one intent on one frame within this long make a single undo step: a drag, a run of
/// arrow-key nudges, or typing.
pub const COALESCE_WINDOW: Duration = Duration::from_millis(600);

/// Typing joins the step before it for this long after the last key, so a pause of a second starts
/// a new step.
pub const TYPING_WINDOW: Duration = Duration::from_millis(1000);

/// Makes the id of a new part of the book, given a short prefix (`sp`, `pg`, `fr`).
pub type IdSource = Box<dyn FnMut(&str) -> String + Send>;

fn random_ids() -> IdSource {
    Box::new(|prefix| {
        let id = uuid::Uuid::new_v4().simple().to_string();
        format!("{prefix}_{}", &id[..12])
    })
}

#[derive(Debug)]
struct Entry {
    before: Delta,
    after: Delta,
    merge: Option<MergeKey>,
    at: Instant,
    /// For an edit to words: the selection before the step and the caret after it, so undo and
    /// redo can put the editor's caret back.
    carets: Option<(EditCaret, EditCaret)>,
}

/// An open book.
pub struct Session {
    book: Book,
    revision: u64,
    saved_revision: u64,
    undo: Vec<Entry>,
    redo: Vec<Entry>,
    ids: IdSource,
    /// Where the caret goes after the last undo or redo, when the step was an edit to words.
    restored: Option<EditCaret>,
}

impl Session {
    pub fn new(book: Book) -> Self {
        Self::with_ids(book, random_ids())
    }

    /// A session for a book brought back after a crash: as unsaved as it can be, so closing it
    /// asks and a save writes it out, though no change has been made to it in this session.
    pub fn recovered(book: Book) -> Self {
        let mut session = Self::new(book);
        session.revision = 1;
        session
    }

    /// A session that makes ids its own way: a counter in tests, for example.
    pub fn with_ids(book: Book, ids: IdSource) -> Self {
        Self {
            book,
            revision: 0,
            saved_revision: 0,
            undo: Vec::new(),
            redo: Vec::new(),
            ids,
            restored: None,
        }
    }

    pub fn book(&self) -> &Book {
        &self.book
    }

    /// The revision the book is at: 0 as opened, and one more for every change, undo and redo.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Whether the book has changed since it was opened or last saved.
    pub fn is_dirty(&self) -> bool {
        self.revision != self.saved_revision
    }

    /// Records that the book as it is now has been saved.
    pub fn mark_saved(&mut self) {
        self.saved_revision = self.revision;
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// How many steps can be undone: a mark to come back to with [`Session::undo`].
    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Where the editor's caret goes after the undo or redo just made, when the step was an edit to
    /// words; `None` when it was anything else, or after a new change.
    pub fn restored_caret(&self) -> Option<&EditCaret> {
        self.restored.as_ref()
    }

    /// Applies an intent now.
    pub fn apply(&mut self, intent: Intent) -> Result<Patch, SessionError> {
        self.apply_at(intent, Instant::now())
    }

    /// Applies an intent as if it arrived at `at`, which decides whether it merges into the last
    /// undo step. A refused intent changes nothing, not even the revision.
    pub fn apply_at(&mut self, intent: Intent, at: Instant) -> Result<Patch, SessionError> {
        if matches!(intent, Intent::EndUndoStep) {
            // Nothing in the book changes: the last step is only closed to what comes next.
            if let Some(last) = self.undo.last_mut() {
                last.merge = None;
            }
            return Ok(Patch {
                revision: self.revision,
                ops: Vec::new(),
            });
        }
        let mut plan = plan(&self.book, &intent, &mut self.ids)?;
        let carets = match &intent {
            Intent::EditText {
                target,
                from,
                to,
                insert,
                ..
            } => {
                let end = folio_model::caret_after(*from, insert);
                Some((
                    EditCaret {
                        target: target.clone(),
                        anchor: *from,
                        focus: *to,
                    },
                    EditCaret {
                        target: target.clone(),
                        anchor: end,
                        focus: end,
                    },
                ))
            }
            _ => None,
        };
        self.restored = None;
        // The spine is as thick as the pages and the paper make it, and the front cover sits past
        // it, so what changes either takes the cover along in the same step.
        let spine_may_move = matches!(
            intent,
            Intent::AddSpread { .. }
                | Intent::AddSpreadWith { .. }
                | Intent::DeleteSpread { .. }
                | Intent::SetPaper { .. }
                | Intent::AddFlowPages { .. }
                | Intent::TrimFlowPages { .. }
        );
        let cover = self
            .book
            .spreads
            .first()
            .filter(|s| s.kind == folio_model::SpreadKind::Cover)
            .map(|s| s.id.clone())
            .filter(|_| spine_may_move);
        let front_before = folio_model::front_offset(&self.book).0;
        let cover_was = cover.as_ref().and_then(|id| {
            self.book
                .spreads
                .iter()
                .find(|s| &s.id == id)
                .cloned()
                .map(|s| (id.clone(), Some(s)))
        });
        let mut before = Delta::capture(&self.book, &plan.touched, &plan.flows);
        run(&mut self.book, intent, &plan, &mut self.ids);
        if let Some(cover) = &cover {
            if slide_front(&mut self.book, cover, front_before) && !plan.touched.contains(cover) {
                plan.touched.push(cover.clone());
                before.spreads.extend(cover_was);
            }
        }
        let after = Delta::capture(&self.book, &plan.touched, &plan.flows);

        let ops = ops_between(&before, &after);
        if ops.is_empty() {
            // The intent changed nothing (a move to where the frame already is).
            return Ok(Patch {
                revision: self.revision,
                ops,
            });
        }

        let merges = plan.merge.is_some()
            && self.redo.is_empty()
            && self
                .undo
                .last()
                .is_some_and(|last| match (&plan.merge, &last.merge) {
                    (Some(now), Some(was)) => {
                        now.joins(was) && at.saturating_duration_since(last.at) <= now.window()
                    }
                    _ => false,
                });
        if merges {
            if let Some(last) = self.undo.last_mut() {
                last.after = after;
                last.at = at;
                last.merge = plan.merge;
                last.carets = match (last.carets.take(), carets) {
                    (Some((before, _)), Some((_, after))) => Some((before, after)),
                    (_, other) => other,
                };
            }
        } else {
            self.undo.push(Entry {
                before,
                after,
                merge: plan.merge,
                at,
                carets,
            });
            if self.undo.len() > UNDO_DEPTH {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
        self.revision += 1;
        Ok(Patch {
            revision: self.revision,
            ops,
        })
    }

    /// Forgets what could be redone: after steps were taken back to undo a batch that failed,
    /// redoing them would bring half of it back.
    pub fn clear_redo(&mut self) {
        self.redo.clear();
    }

    /// Makes the last `count` undo steps one, so a change and what had to follow from it (pages
    /// added for a story that grew) are taken back together. Does nothing when there are fewer.
    pub fn join_last(&mut self, count: usize) {
        if count < 2 || self.undo.len() < count {
            return;
        }
        let mut steps = self.undo.split_off(self.undo.len() - count).into_iter();
        let first = steps.next().expect("count is at least two");
        let joined = steps.fold(first, |so_far, next| Entry {
            before: so_far.before.before_both(&next.before),
            after: so_far.after.after_both(&next.after),
            merge: None,
            at: next.at,
            carets: so_far.carets,
        });
        self.undo.push(joined);
    }

    /// Undoes the last step, or `None` when there is nothing to undo.
    pub fn undo(&mut self) -> Option<Patch> {
        let entry = self.undo.pop()?;
        let ops = entry.before.restore(&mut self.book);
        self.restored = entry.carets.as_ref().map(|(before, _)| before.clone());
        self.redo.push(entry);
        self.revision += 1;
        Some(Patch {
            revision: self.revision,
            ops,
        })
    }

    /// Redoes the last undone step, or `None` when there is nothing to redo.
    pub fn redo(&mut self) -> Option<Patch> {
        let mut entry = self.redo.pop()?;
        let ops = entry.after.restore(&mut self.book);
        self.restored = entry.carets.as_ref().map(|(_, after)| after.clone());
        // A step redone is closed: the next intent starts a new one.
        entry.merge = None;
        self.undo.push(entry);
        self.revision += 1;
        Some(Patch {
            revision: self.revision,
            ops,
        })
    }
}

/// Moves what sits on the front half of the cover along with the front, when the spine has grown or
/// shrunk since the front was at `front_before`: frames centred past the middle of the old spine
/// slide by the change, and the back and the spine's own frames stay where they are. Whether any
/// frame moved.
fn slide_front(
    book: &mut folio_model::Book,
    cover: &folio_model::SpreadId,
    front_before: f64,
) -> bool {
    let change = folio_model::front_offset(book).0 - front_before;
    if change.abs() < 1e-9 {
        return false;
    }
    let middle = (front_before + book.trim.w.0) / 2.0;
    let mut moved = false;
    if let Some(spread) = book.spreads.iter_mut().find(|s| &s.id == cover) {
        for frame in &mut spread.frames {
            let base = frame.base_mut();
            if base.rect.x.0 + base.rect.w.0 / 2.0 > middle {
                base.rect.x.0 += change;
                moved = true;
            }
        }
    }
    moved
}

/// What changed between two snapshots of the same touched spreads.
fn ops_between(before: &Delta, after: &Delta) -> Vec<PatchOp> {
    let mut ops = Vec::new();
    for (id, now) in &after.spreads {
        let was = before
            .spreads
            .iter()
            .find(|(b, _)| b == id)
            .and_then(|(_, s)| s.as_ref());
        match (was, now) {
            (Some(was), Some(now)) if was == now => {}
            (_, Some(now)) => ops.push(PatchOp::SetSpread {
                spread: now.clone(),
            }),
            (Some(_), None) => ops.push(PatchOp::RemoveSpread { id: id.clone() }),
            (None, None) => {}
        }
    }
    if before.order != after.order {
        ops.push(PatchOp::SetOrder {
            order: after.order.clone(),
        });
    }
    if before.masters != after.masters {
        ops.push(after.masters.op());
    }
    if before.language != after.language {
        ops.push(PatchOp::SetLanguage {
            language: after.language.clone(),
        });
    }
    if before.barcode != after.barcode {
        ops.push(PatchOp::SetBarcode {
            choice: after.barcode.clone(),
        });
    }
    if before.paper != after.paper {
        ops.push(PatchOp::SetPaper { paper: after.paper });
    }
    for (id, now) in &after.flows {
        let was = before
            .flows
            .iter()
            .find(|(b, _)| b == id)
            .and_then(|(_, f)| f.as_ref());
        match (was, now) {
            (Some(was), Some(now)) if was == now => {}
            (_, Some(now)) => ops.push(PatchOp::SetFlow { flow: now.clone() }),
            (Some(_), None) => ops.push(PatchOp::RemoveFlow { id: id.clone() }),
            (None, None) => {}
        }
    }
    if before.keywords != after.keywords {
        ops.push(PatchOp::SetKeywords {
            keywords: after.keywords.clone(),
        });
    }
    if before.styles != after.styles {
        ops.push(PatchOp::SetStyles {
            styles: after.styles.clone(),
        });
    }
    if before.details != after.details {
        ops.push(PatchOp::SetDetails {
            details: after.details.clone(),
        });
    }
    if before.preflight_ignore != after.preflight_ignore {
        ops.push(PatchOp::SetPreflightIgnore {
            keys: after.preflight_ignore.clone(),
        });
    }
    if before.fonts != after.fonts {
        ops.push(PatchOp::SetFonts {
            fonts: after.fonts.clone(),
        });
    }
    if before.export != after.export {
        ops.push(PatchOp::SetExportPrefs {
            prefs: Box::new(after.export.clone()),
        });
    }
    if before.proofing != after.proofing {
        ops.push(PatchOp::SetProofing {
            proofing: after.proofing.clone(),
        });
    }
    if before.geometry != after.geometry {
        ops.push(PatchOp::SetGeometry {
            trim: after.geometry.trim,
            margins: after.geometry.margins,
            bleed: after.geometry.bleed,
            grid: after.geometry.grid.clone(),
        });
    }
    ops
}
