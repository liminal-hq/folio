// Tests for the rulers on the stage: shown when on, with a corner button that picks the unit
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
		{ type: 'text', id: 't1', box: { x: 10, y: 20, w: 40, h: 30 }, rotation: 0, locked: false },
	],
	guides: [],
	masterOverrides: [],
} as unknown as Spread;

const stage = (rulers: { on: boolean; unit: 'inches' | 'picas' }, extra = {}) => (
	<Stage book={book} spread={spread} label="Cover" layout={null} rulers={rulers} {...extra} />
);

describe('Stage rulers', () => {
	it('shows a ruler along the top and one down the left only when they are on', () => {
		const { container, rerender } = render(stage({ on: false, unit: 'inches' }));
		expect(container.querySelector('[data-ruler]')).toBeNull();
		rerender(stage({ on: true, unit: 'inches' }));
		expect(container.querySelector('[data-ruler="x"]')).not.toBeNull();
		expect(container.querySelector('[data-ruler="y"]')).not.toBeNull();
	});

	it('tints the selected frame on both rulers', () => {
		const { container } = render(stage({ on: true, unit: 'inches' }, { selection: ['t1'] }));
		expect(container.querySelectorAll('[data-extent]')).toHaveLength(2);
	});

	it('names the unit on the corner button and changes it from its menu', () => {
		const onRulerUnit = vi.fn();
		render(stage({ on: true, unit: 'picas' }, { onRulerUnit }));
		fireEvent.click(screen.getByRole('button', { name: 'Ruler units' }));
		expect(screen.getByRole('button', { name: 'Ruler units' })).toHaveTextContent('pc');
		fireEvent.click(screen.getByText('Millimetres'));
		expect(onRulerUnit).toHaveBeenCalledWith('millimetres');
	});
});
