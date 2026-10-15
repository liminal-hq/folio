// Tests for the page shelf: choosing, adding, the thumbnail menu and dragging to reorder
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { expectEveryItemHasIcon } from '@liminal-hq/folio-chrome/ContextMenu/expectEveryItemHasIcon';
import { fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { Shelf } from './Shelf';
import { shelfIntent, shelfMenuItems } from './shelfMenu';

const spread = (id: string, kind: string, pages = 1) => ({
	id,
	kind,
	pages: Array.from({ length: pages }, (_, i) => ({ id: `${id}-${i}`, master: null })),
	frames: [],
});

const book = {
	trim: { w: 100, h: 100 },
	bleed: 0,
	margins: { top: 0, bottom: 0, inside: 0, outside: 0 },
	spreads: [
		spread('cover', 'cover'),
		spread('a', 'opener'),
		spread('b', 'spread', 2),
		spread('c', 'closer'),
	],
} as unknown as Book;

const labels = ['Cover', 'Page 1', 'Pages 2–3', 'Page 4'];

function setup() {
	const props = {
		onSelect: vi.fn(),
		onAdd: vi.fn(),
		onCommand: vi.fn(),
		onMove: vi.fn(),
	};
	render(<Shelf book={book} labels={labels} selected={1} {...props} />);
	return props;
}

describe('the shelf', () => {
	it('selects a spread, and adds pages at the end', async () => {
		const p = setup();
		await userEvent.click(screen.getByRole('button', { name: /Pages 2–3/ }));
		expect(p.onSelect).toHaveBeenCalledWith(2);
		await userEvent.click(screen.getByRole('button', { name: 'Add pages' }));
		expect(p.onAdd).toHaveBeenCalled();
	});

	it('opens the menu on right-click and runs a command on that spread', async () => {
		const p = setup();
		fireEvent.contextMenu(screen.getByRole('button', { name: /Pages 2–3/ }));
		expect(p.onSelect).toHaveBeenCalledWith(2);
		const menu = await screen.findByRole('menu', { name: 'Spread' });
		await userEvent.click(within(menu).getByRole('menuitem', { name: 'Duplicate' }));
		expect(p.onCommand).toHaveBeenCalledWith('duplicate', 2);
	});

	it('reorders by dragging one thumbnail onto another, never the cover', () => {
		const p = setup();
		const items = screen.getAllByRole('listitem');
		const data = { setData: vi.fn() };
		fireEvent.dragStart(items[1]!, { dataTransfer: data });
		fireEvent.dragOver(items[3]!, { dataTransfer: data });
		fireEvent.drop(items[3]!, { dataTransfer: data });
		expect(p.onMove).toHaveBeenCalledWith(1, 3);
		expect(items[0]).toHaveAttribute('draggable', 'false');
		fireEvent.dragStart(items[2]!, { dataTransfer: data });
		fireEvent.drop(items[0]!, { dataTransfer: data });
		expect(p.onMove).toHaveBeenCalledTimes(1);
	});
});

describe('the shelf menu', () => {
	it('gives every row an icon', () => {
		expectEveryItemHasIcon(shelfMenuItems(book, 2));
	});

	it('keeps the cover where it is and the first page from moving onto it', () => {
		const cover = shelfMenuItems(book, 0).filter((i) => i.type === 'action');
		expect(cover.every((i) => i.type === 'action' && i.disabled)).toBe(true);
		const first = shelfMenuItems(book, 1).find((i) => i.id === 'moveLeft');
		expect(first).toMatchObject({ disabled: true });
		expect(shelfMenuItems(book, 3).find((i) => i.id === 'moveRight')).toMatchObject({
			disabled: true,
		});
	});

	it('makes the intents for each command', () => {
		expect(shelfIntent('moveRight', book, 2)).toEqual({ type: 'MoveSpread', spread: 'b', to: 3 });
		expect(shelfIntent('moveLeft', book, 2)).toEqual({ type: 'MoveSpread', spread: 'b', to: 1 });
		expect(shelfIntent('moveLeft', book, 1)).toBeNull();
		expect(shelfIntent('duplicate', book, 2)).toEqual({
			type: 'AddSpread',
			after: 'b',
			duplicate: true,
		});
		expect(shelfIntent('remove', book, 2)).toEqual({ type: 'DeleteSpread', spread: 'b' });
		expect(shelfIntent('remove', book, 0)).toBeNull();
	});
});

describe('shapes in a thumbnail', () => {
	it('are drawn with their own fill and outline, not as grey boxes', () => {
		const withShape = {
			...book,
			spreads: [
				{
					...spread('a', 'opener'),
					frames: [
						{
							type: 'shape',
							id: 's',
							box: { x: 0, y: 0, w: 50, h: 20 },
							rotation: 0,
							locked: false,
							shape: 'ellipse',
							fill: '#112233',
							stroke: '#445566',
							style: null,
						},
					],
				},
			],
		} as unknown as Book;
		const { container } = render(
			<Shelf book={withShape} labels={['Page 1']} selected={0} onSelect={() => undefined} />,
		);
		const ellipse = container.querySelector('svg ellipse');
		expect(ellipse?.getAttribute('fill')).toBe('#112233');
		expect(ellipse?.getAttribute('stroke')).toBe('#445566');
	});
});
