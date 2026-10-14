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
		expect(frameIntents('duplicate', 's', frame(false))).toEqual([
			{ type: 'DuplicateFrame', spread: 's', frame: 'f' },
		]);
		expect(frameIntents('front', 's', frame(false))).toEqual([
			{ type: 'Reorder', spread: 's', frame: 'f', to: 'front' },
		]);
		expect(frameIntents('unlock', 's', frame(true))).toEqual([
			{ type: 'LockFrame', spread: 's', frame: 'f', locked: false },
		]);
	});
	it('does not delete a locked frame', () => {
		expect(frameIntents('delete', 's', frame(true))).toEqual([]);
		expect(frameIntents('delete', 's', frame(false))).toHaveLength(1);
	});
});
