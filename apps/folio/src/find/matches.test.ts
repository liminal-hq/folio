// Tests for which matches belong on a page and which frames show them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FindMatch } from '@liminal-hq/folio-protocol/generated/FindMatch';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import { describe, expect, it } from 'vitest';
import { layoutOf } from '../text/fixtureLayout';
import { framesShowing, laidOut, matchKey, onShown, relocate, stepIndex } from './matches';

const match = (over: Partial<FindMatch> = {}): FindMatch => ({
	target: { type: 'frame', frame: 'a' },
	para: 0,
	from: 0,
	to: 3,
	spread: 's',
	context: 'cat',
	locked: false,
	...over,
});

describe('matchKey', () => {
	it('names a match by what an edit takes', () => {
		expect(matchKey(match())).toBe(matchKey(match({ context: 'other words' })));
		expect(matchKey(match())).not.toBe(matchKey(match({ from: 1 })));
		expect(matchKey(match())).not.toBe(matchKey(match({ target: { type: 'shape', frame: 'a' } })));
	});
});

describe('onShown', () => {
	it('keeps the matches of the spread being shown, or of the master being edited', () => {
		expect(onShown(match(), 's', null)).toBe(true);
		expect(onShown(match(), 'other', null)).toBe(false);
		const onMaster = match({ spread: undefined, master: 'm' });
		expect(onShown(onMaster, 's', null)).toBe(false);
		expect(onShown(onMaster, 'm', 'm')).toBe(true);
		expect(onShown(match(), 's', 'm')).toBe(false);
	});
});

describe('framesShowing', () => {
	const frames = [
		{ type: 'text', id: 'a', flow: null },
		{ type: 'text', id: 'b', flow: { id: 'fl', index: 0 } },
		{ type: 'text', id: 'c', flow: { id: 'fl', index: 1 } },
		{ type: 'text', id: 'm', flow: { id: 'fl', index: 2 }, fromMaster: 'x' },
		{ type: 'table', id: 't' },
	] as unknown as Frame[];

	it('finds a frame, a shape or a cell by its own words', () => {
		expect(framesShowing(match(), frames).map((s) => s.layoutKey)).toEqual(['a']);
		expect(
			framesShowing(match({ target: { type: 'cell', frame: 't', row: 1, col: 2 } }), frames).map(
				(s) => s.layoutKey,
			),
		).toEqual(['t#1.2']);
	});

	it('offers every frame of a story on the spread, apart from master ones', () => {
		const shown = framesShowing(match({ target: { type: 'story', flow: 'fl' } }), frames);
		expect(shown.map((s) => s.layoutKey)).toEqual(['b', 'c']);
	});
});

describe('stepIndex', () => {
	it('goes round the ends and starts from the first or last', () => {
		expect(stepIndex(null, 3, false)).toBe(0);
		expect(stepIndex(null, 3, true)).toBe(2);
		expect(stepIndex(2, 3, false)).toBe(0);
		expect(stepIndex(0, 3, true)).toBe(2);
		expect(stepIndex(1, 3, false)).toBe(2);
		expect(stepIndex(0, 0, false)).toBeNull();
	});
});

describe('relocate', () => {
	const list = [match({ from: 0, to: 3 }), match({ from: 5, to: 8 }), match({ from: 9, to: 12 })];

	it('keeps the same match, else the one now at its place, else the last', () => {
		expect(relocate(list, { key: matchKey(list[1]!), index: 0 })).toBe(1);
		expect(relocate(list, { key: 'gone', index: 1 })).toBe(1);
		expect(relocate(list, { key: 'gone', index: 9 })).toBe(2);
		expect(relocate(list, null)).toBe(0);
		expect(relocate([], null)).toBeNull();
	});
});

describe('laidOut', () => {
	it('is true when a line of the layout holds some of the words', () => {
		const layout = layoutOf([['one two', 'three']]);
		expect(laidOut(layout, match({ para: 0, from: 8, to: 13 }))).toBe(true);
		expect(laidOut(layout, match({ para: 1, from: 0, to: 3 }))).toBe(false);
		expect(laidOut(undefined, match())).toBe(false);
	});
});
