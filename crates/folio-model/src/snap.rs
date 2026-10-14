// Snapping maths shared by the canvas and the layout engine
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// ── Snapping (shared maths; mirrors Spindle SceneCanvas, extended for books) ──
pub const SNAP_THRESHOLD_PX: f64 = 8.0;

/// Returns (shift, target) for the closest candidate/target pair within `threshold`.
pub fn snap_axis(candidates: &[f64], targets: &[f64], threshold: f64) -> Option<(f64, f64)> {
    let mut best: Option<(f64, f64, f64)> = None; // (dist, shift, target)
    for &c in candidates {
        for &t in targets {
            let d = (t - c).abs();
            if d <= threshold && best.is_none_or(|b| d < b.0) {
                best = Some((d, t - c, t));
            }
        }
    }
    best.map(|(_, s, t)| (s, t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snaps_to_the_nearest_target_within_the_threshold() {
        assert_eq!(
            snap_axis(&[10.0, 15.0], &[0.0, 16.0], 2.0),
            Some((1.0, 16.0))
        );
    }

    #[test]
    fn nothing_snaps_outside_the_threshold() {
        assert_eq!(snap_axis(&[10.0], &[20.0], 2.0), None);
    }

    #[test]
    fn no_candidates_or_no_targets_is_no_snap() {
        assert_eq!(snap_axis(&[], &[1.0], 2.0), None);
        assert_eq!(snap_axis(&[1.0], &[], 2.0), None);
    }

    #[test]
    fn a_frame_edge_already_on_a_target_does_not_move() {
        assert_eq!(snap_axis(&[36.0, 100.0], &[36.0], 8.0), Some((0.0, 36.0)));
    }
}
