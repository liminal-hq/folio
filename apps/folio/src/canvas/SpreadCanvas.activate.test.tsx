// Tests for double-clicking a frame something else takes, and for the grey hint drawn in a box
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { SpreadCanvas } from './SpreadCanvas';

const box = { x: 20, y: 20, w: 144, h: 86 };
const spread = {
	id: 's',
	kind: 'single',
	pages: [{ id: 'p', master: null }],
	frames: [
		{
			type: 'shape',
			id: 'cover_barcode',
			box,
			rotation: 0,
			locked: true,
			shape: 'rect',
			fill: '#ffffff',
			style: null,
		},
	],
	guides: [],
	masterOverrides: [],
} as unknown as Spread;
const book = {
	trim: { w: 400, h: 400 },
	bleed: 0,
	margins: { top: 0, bottom: 0, inside: 0, outside: 0 },
	grid: { columns: 1, gutter: 0, baseline: null },
	masterPages: [],
} as unknown as Book;

describe('a frame something else takes', () => {
	it('is handed to the one that wants it when it is double-clicked', () => {
		const onActivate = vi.fn(() => true);
		const onEditText = vi.fn();
		render(
			<SpreadCanvas
				book={book}
				spread={spread}
				layout={null}
				zoom={1}
				label="Page"
				selection={[]}
				onSelect={() => undefined}
				onEditText={onEditText}
				onActivate={onActivate}
			/>,
		);
		fireEvent.doubleClick(document.querySelector('[data-hit="cover_barcode"]')!);
		expect(onActivate).toHaveBeenCalledWith('cover_barcode');
		expect(onEditText).not.toHaveBeenCalled();
	});

	it('shows a hint in grey over the box, which takes no clicks', () => {
		render(
			<SpreadCanvas
				book={book}
				spread={spread}
				layout={null}
				zoom={1}
				label="Page"
				hint={{ box, text: 'Double-click to set up the barcode' }}
			/>,
		);
		const hint = screen.getByTestId('canvas-hint');
		expect(hint).toHaveTextContent('Double-click to set up the barcode');
		expect(hint).toHaveAttribute('pointer-events', 'none');
	});
});
