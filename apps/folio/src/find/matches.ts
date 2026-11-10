// Which matches belong on a spread, and which frames show them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FindMatch } from '@liminal-hq/folio-protocol/generated/FindMatch';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import { cellId } from '../canvas/tableCells';

/** A match named by what an edit takes, so it can be found again after the book changes. */
export function matchKey(match: Pick<FindMatch, 'target' | 'para' | 'from' | 'to'>): string {
	return JSON.stringify([match.target, match.para, match.from, match.to]);
}

/** Whether a match is on the page being shown: a spread, or the master page being edited. */
export function onShown(match: FindMatch, spread: string, master: string | null): boolean {
	return master === null ? match.spread === spread : match.master === master;
}

/** The frame of a match and the key its words are laid out under, for each frame that shows it. */
export interface Shown {
	frame: Frame;
	layoutKey: string;
}

/**
 * The frames of `frames` that can show a match: a frame's, a shape's or a cell's own words are in
 * that one; a story's are in any of its frames on the spread (the layout says which holds them).
 */
export function framesShowing(match: FindMatch, frames: readonly Frame[]): Shown[] {
	const target = match.target;
	switch (target.type) {
		case 'frame':
		case 'shape':
			return frames
				.filter((f) => f.id === target.frame)
				.map((frame) => ({ frame, layoutKey: frame.id }));
		case 'cell':
			return frames
				.filter((f) => f.id === target.frame)
				.map((frame) => ({
					frame,
					layoutKey: cellId({ frame: frame.id, row: target.row, col: target.col }),
				}));
		case 'story':
			return frames
				.filter((f) => f.type === 'text' && f.flow?.id === target.flow && !f.fromMaster)
				.map((frame) => ({ frame, layoutKey: frame.id }));
	}
}

/** The index after (or before) `at`, going round the ends; `null` when there is none. */
export function stepIndex(at: number | null, total: number, back: boolean): number | null {
	if (total === 0) return null;
	if (at === null) return back ? total - 1 : 0;
	return (at + (back ? total - 1 : 1)) % total;
}

/**
 * Where the current match is in a fresh list of matches: the same match when it is still there,
 * else the one now at its place (a match was replaced), else the last.
 */
export function relocate(
	matches: readonly FindMatch[],
	previous: { key: string; index: number } | null,
): number | null {
	if (matches.length === 0) return null;
	if (!previous) return 0;
	const same = matches.findIndex((m) => matchKey(m) === previous.key);
	return same >= 0 ? same : Math.min(previous.index, matches.length - 1);
}

/** What the page draws of a search: the matches, the current one, and the one to select now. */
export interface FindDisplay {
	matches: readonly FindMatch[];
	current: FindMatch | null;
	focus: { match: FindMatch; nonce: number } | null;
}

/** Whether a frame's laid-out lines hold any of a match's words. */
export function laidOut(layout: FrameLayout | undefined, match: FindMatch): boolean {
	return (
		layout?.lines.some(
			(line) => line.paragraph === match.para && line.start < match.to && line.end > match.from,
		) ?? false
	);
}
