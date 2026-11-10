// Tests for the frame menu's rows and the intents its commands make
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { expectEveryItemHasIcon } from '@liminal-hq/folio-chrome/ContextMenu/expectEveryItemHasIcon';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import { describe, expect, it } from 'vitest';
import { frameIntents, frameMenuItems } from './frameMenu';

const frame = (locked: boolean) =>
	({ type: 'shape', id: 'f', box: { x: 0, y: 0, w: 1, h: 1 }, locked }) as unknown as Frame;

describe('frameMenuItems', () => {
	it('gives every row an icon', () => {
		expectEveryItemHasIcon(frameMenuItems([frame(false)]));
	});
	it('offers Cut, Copy and Paste first, and cannot cut what is locked', () => {
		const free = frameMenuItems([frame(false)]);
		expect(free.slice(0, 3).map((i) => i.id)).toEqual(['cut', 'copy', 'paste']);
		expect(free[0]).toMatchObject({ shortcut: 'Ctrl+X' });
		expect(free.find((i) => i.id === 'cut')).not.toHaveProperty('disabled', true);
		expect(frameMenuItems([frame(true)]).find((i) => i.id === 'cut')).toMatchObject({
			disabled: true,
		});
		expect(frameMenuItems([frame(true)]).find((i) => i.id === 'copy')).not.toHaveProperty(
			'disabled',
			true,
		);
	});
	it('offers Group for several free frames, and Ungroup for one group', () => {
		const two = frameMenuItems([frame(false), { ...frame(false), id: 'g' } as Frame]);
		expect(two.find((i) => i.id === 'group')).toMatchObject({ shortcut: 'Ctrl+G' });
		expect(two.map((i) => i.id)).not.toContain('ungroup');
		expect(frameMenuItems([frame(false)]).map((i) => i.id)).not.toContain('group');
		const group = { ...frame(false), type: 'group', children: [] } as unknown as Frame;
		const one = frameMenuItems([group]);
		expect(one.find((i) => i.id === 'ungroup')).toMatchObject({ shortcut: 'Ctrl+Shift+G' });
		// Not on a master page, and not with a locked frame.
		expect(frameMenuItems([frame(false), frame(false)], true).map((i) => i.id)).not.toContain(
			'group',
		);
		expect(frameMenuItems([frame(false), frame(true)]).map((i) => i.id)).not.toContain('group');
	});
	it('offers Lock for free frames and Unlock for locked ones, which cannot be deleted', () => {
		const free = frameMenuItems([frame(false)]);
		expect(free.map((i) => i.id)).toContain('lock');
		const locked = frameMenuItems([frame(true)]);
		expect(locked.map((i) => i.id)).toContain('unlock');
		expect(locked.find((i) => i.id === 'delete')).toMatchObject({ disabled: true });
	});
});

describe('frameIntents', () => {
	it('makes one intent per command', () => {
		expect(frameIntents('duplicate', { spread: 's' }, frame(false))).toEqual([
			{ type: 'DuplicateFrame', on: { spread: 's' }, frame: 'f' },
		]);
		expect(frameIntents('front', { spread: 's' }, frame(false))).toEqual([
			{ type: 'Reorder', on: { spread: 's' }, frame: 'f', to: 'front' },
		]);
		expect(frameIntents('unlock', { spread: 's' }, frame(true))).toEqual([
			{ type: 'LockFrame', on: { spread: 's' }, frame: 'f', locked: false },
		]);
	});
	it('leaves the clipboard commands to the book view, which has the frames together', () => {
		for (const command of ['cut', 'copy', 'paste'] as const) {
			expect(frameIntents(command, { spread: 's' }, frame(false))).toEqual([]);
		}
	});
	it('does not delete a locked frame', () => {
		expect(frameIntents('delete', { spread: 's' }, frame(true))).toEqual([]);
		expect(frameIntents('delete', { spread: 's' }, frame(false))).toHaveLength(1);
	});
});
