// Tests for when frames can be grouped
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import { describe, expect, it } from 'vitest';
import { canGroup, canUngroup, childIds, groupIntent, ungroupIntent } from './grouping';

const frame = (id: string, extra = {}) =>
	({
		type: 'shape',
		id,
		box: { x: 0, y: 0, w: 10, h: 10 },
		rotation: 0,
		locked: false,
		...extra,
	}) as unknown as Frame;
const group = (extra = {}) =>
	frame('g', { type: 'group', children: [frame('a'), frame('b')], ...extra });

describe('canGroup', () => {
	it('needs two or more free frames on a page', () => {
		expect(canGroup([frame('a'), frame('b')], false)).toBe(true);
		expect(canGroup([frame('a')], false)).toBe(false);
		expect(canGroup([], false)).toBe(false);
		expect(canGroup([frame('a'), frame('b')], true)).toBe(false);
	});

	it('refuses locked frames, a master’s, and a story’s', () => {
		expect(canGroup([frame('a'), frame('b', { locked: true })], false)).toBe(false);
		expect(canGroup([frame('a'), frame('b', { fromMaster: 'mst_a' })], false)).toBe(false);
		const flowed = frame('c', { type: 'text', flow: { id: 'fl', index: 0 } });
		expect(canGroup([frame('a'), flowed], false)).toBe(false);
		expect(canGroup([frame('a'), frame('c', { type: 'text' })], false)).toBe(true);
	});

	it('lets groups be grouped with frames', () => {
		expect(canGroup([group(), frame('c')], false)).toBe(true);
	});
});

describe('canUngroup', () => {
	it('takes one free group apart and nothing else', () => {
		expect(canUngroup([group()])).toBe(true);
		expect(canUngroup([group({ locked: true })])).toBe(false);
		expect(canUngroup([frame('a')])).toBe(false);
		expect(canUngroup([group(), group()])).toBe(false);
		expect(canUngroup([])).toBe(false);
	});
});

describe('the intents', () => {
	const on = { spread: 'sp' } as never;
	it('names the frames, and the group', () => {
		expect(groupIntent([frame('a'), frame('b')], on)).toEqual({
			type: 'GroupFrames',
			on,
			frames: ['a', 'b'],
		});
		expect(ungroupIntent(group(), on)).toEqual({ type: 'UngroupFrames', on, frame: 'g' });
		expect(childIds(group())).toEqual(['a', 'b']);
		expect(childIds(frame('a'))).toEqual([]);
	});
});
