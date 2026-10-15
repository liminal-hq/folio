// Tests for the frames the page menu adds, and the menu itself
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { expectEveryItemHasIcon } from '@liminal-hq/folio-chrome/ContextMenu/expectEveryItemHasIcon';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { boxAt, newFrame, newFrameId } from './newFrames';
import { pageMenuItems } from './pageMenu';
import { DEFAULT_MARKS } from './marksModel';
import { SpreadCanvas } from './SpreadCanvas';

const page = { x: 0, y: 0, w: 400, h: 400 };

describe('boxAt', () => {
	it('centres the new frame where it was asked for', () => {
		expect(boxAt('text', { x: 200, y: 200 }, page)).toEqual({ x: 100, y: 160, w: 200, h: 80 });
	});
	it('keeps it inside the page near an edge, and shrinks it to a small page', () => {
		expect(boxAt('picture', { x: 5, y: 395 }, page)).toEqual({ x: 0, y: 220, w: 240, h: 180 });
		expect(boxAt('picture', { x: 50, y: 50 }, { x: 0, y: 0, w: 100, h: 100 })).toEqual({
			x: 0,
			y: 0,
			w: 100,
			h: 100,
		});
	});
});

describe('newFrame', () => {
	it('makes a text box in the given style with words to replace', () => {
		const frame = newFrame('text', page, 'body') as unknown as Record<string, unknown>;
		expect(frame).toMatchObject({ type: 'text', style: 'body', columns: 1, locked: false });
		expect(JSON.stringify(frame.content)).toContain('Type your words here');
	});
	it('makes a picture placeholder and a note box', () => {
		expect(newFrame('picture', page, 'body')).toMatchObject({
			type: 'image',
			image: { asset: null },
		});
		expect(newFrame('note', page, 'body')).toMatchObject({ type: 'shape', shape: 'round' });
	});
	it('gives every frame its own id', () => {
		const ids = new Set(Array.from({ length: 50 }, newFrameId));
		expect(ids.size).toBe(50);
		expect([...ids][0]).toMatch(/^fr_[0-9a-f]{12}$/);
	});
});

describe('pageMenuItems', () => {
	it('gives every row an icon, and shows the helpers as they are', () => {
		const items = pageMenuItems(DEFAULT_MARKS, true);
		expectEveryItemHasIcon(items.filter((i) => i.type !== 'checkbox'));
		expect(items.find((i) => i.id === 'toggleMargins')).toMatchObject({ checked: true });
		expect(items.find((i) => i.id === 'toggleGrid')).toMatchObject({ checked: false });
	});
	it('offers no more pages, and no master items to hide, on a master page', () => {
		const item = { page: 'p1', frame: 'fr_pn_a', label: 'Page number', hidden: false };
		const items = pageMenuItems(DEFAULT_MARKS, true, [item], true);
		expect(items.some((i) => i.id === 'addPages' || i.id === 'duplicateSpread')).toBe(false);
		expect(items.some((i) => String(i.id).startsWith('hideMaster'))).toBe(false);
		expect(items.some((i) => i.id === 'toggleMargins')).toBe(true);
		expectEveryItemHasIcon(items.filter((i) => i.type !== 'checkbox'));
		expect(
			pageMenuItems(DEFAULT_MARKS, true, [item]).some((i) => i.id === 'hideMaster:p1:fr_pn_a'),
		).toBe(true);
	});
	it('does not offer to duplicate the cover', () => {
		expect(
			pageMenuItems(DEFAULT_MARKS, false).find((i) => i.id === 'duplicateSpread'),
		).toMatchObject({
			disabled: true,
		});
	});
});

describe('the bare page', () => {
	it('asks for the menu with the spot in points', () => {
		const book = {
			trim: { w: 400, h: 400 },
			bleed: 10,
			margins: { top: 0, bottom: 0, inside: 0, outside: 0 },
			grid: { columns: 1, gutter: 0, baseline: null },
		} as unknown as Book;
		const spread = {
			id: 's',
			kind: 'single',
			pages: [{ id: 'p', master: null }],
			frames: [],
			guides: [],
			masterOverrides: [],
		} as unknown as Spread;
		const onPageMenu = vi.fn();
		render(
			<SpreadCanvas
				book={book}
				spread={spread}
				layout={null}
				zoom={1}
				label="Page"
				selection={[]}
				onSelect={() => {}}
				onPageMenu={onPageMenu}
			/>,
		);
		const svg = screen.getByRole('group', { name: 'Page' });
		svg.getBoundingClientRect = () => ({ left: 100, top: 50 }) as DOMRect;
		fireEvent.contextMenu(svg.querySelector('rect')!, {
			clientX: 100 + 20 * (96 / 72),
			clientY: 50,
		});
		const [, point] = onPageMenu.mock.calls[0]!;
		// 20 points in from the left edge of the bleed, which starts 10 points before the trim.
		expect(point.x).toBeCloseTo(-10 + 20);
		expect(point.y).toBeCloseTo(-10);
	});
});

describe('line and bubble frames', () => {
	const page = { x: 0, y: 0, w: 300, h: 300 };

	it('starts a line flat, in its outline colour and without a fill', () => {
		const box = boxAt('line', { x: 150, y: 150 }, page);
		expect(box).toMatchObject({ w: 144, h: 0 });
		expect(newFrame('line', box, 'body')).toMatchObject({
			type: 'shape',
			shape: 'line',
			fill: null,
			stroke: '#14110f',
			strokeWidth: 2,
		});
	});

	it.each(['speech', 'thought', 'shout'] as const)(
		'starts a %s bubble white with an outline',
		(kind) => {
			const frame = newFrame(kind, boxAt(kind, { x: 150, y: 150 }, page), 'body');
			expect(frame).toMatchObject({
				type: 'shape',
				shape: kind,
				fill: '#ffffff',
				stroke: '#14110f',
			});
		},
	);
});
