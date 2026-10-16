// Tests for what lets go of the selection on the stage, and what does not
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Stage } from './Stage';

const book = {
	trim: { w: 100, h: 100 },
	bleed: 9,
	margins: { top: 10, bottom: 10, inside: 10, outside: 10 },
	grid: { columns: 1, gutter: 6, baseline: { increment: 12, offset: 0 } },
} as unknown as Book;

const spread = {
	id: 's1',
	kind: 'opener',
	pages: [{ id: 'p1', master: null }],
	frames: [
		{
			type: 'text',
			id: 't1',
			box: { x: 10, y: 20, w: 40, h: 30 },
			rotation: 0,
			locked: false,
			style: 'body',
			columns: 1,
			baselineLock: false,
			content: { inline: [{ type: 'run', text: 'Down by the reeds' }] },
		},
	],
	guides: [],
	masterOverrides: [],
} as unknown as Spread;

describe('Stage selection', () => {
	it('keeps the frame selected while words in it are picked, and lets go on the bare table', () => {
		const onSelect = vi.fn();
		const { container } = render(
			<Stage
				book={book}
				spread={spread}
				label="Cover"
				layout={null}
				selection={['t1']}
				onSelect={onSelect}
				onEditText={() => {}}
			/>,
		);
		fireEvent.doubleClick(container.querySelector('[data-hit="t1"]')!);
		const area = screen.getByRole('textbox');
		onSelect.mockClear();

		fireEvent.pointerDown(area, { button: 0 });
		expect(onSelect).not.toHaveBeenCalled();

		fireEvent.pointerDown(container.querySelector('[class*="table"]')!, { button: 0 });
		expect(onSelect).toHaveBeenCalledWith([]);
	});
});
