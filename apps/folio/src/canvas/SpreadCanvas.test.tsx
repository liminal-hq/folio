// Tests for the spread canvas: what it draws from a book and the lines Rust positioned
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { DEFAULT_MARKS, SpreadCanvas } from './SpreadCanvas';

const book = {
	trim: { w: 100, h: 100 },
	bleed: 9,
	margins: { top: 10, bottom: 10, inside: 10, outside: 10 },
	grid: { columns: 2, gutter: 6, baseline: { increment: 12, offset: 0 } },
} as unknown as Book;

const rect = { x: 10, y: 10, w: 80, h: 40 };
const spread = {
	id: 's1',
	kind: 'opener',
	pages: [{ id: 'p1', master: null }],
	frames: [
		{ type: 'text', id: 't1', box: rect, rotation: 0, locked: false },
		{ type: 'image', id: 'i1', box: { x: 0, y: 50, w: 100, h: 50 }, placeholderLabel: 'Otter' },
	],
	guides: [],
	masterOverrides: [],
} as unknown as Spread;

const layout: SpreadLayout = {
	spread: 's1',
	preflight: [],
	frames: [
		{
			frame: 't1',
			overset: false,
			lines: [
				{
					x: 0,
					top: 0,
					baseline: 10,
					width: 60,
					height: 14,
					runs: [
						{
							text: 'Once upon a',
							x: 0,
							width: 50,
							family: 'Literata',
							size: 12,
							weight: 400,
							italic: false,
							tracking: 0,
							hyphen: false,
						},
						{
							text: 'tim',
							x: 52,
							width: 8,
							family: 'Literata',
							size: 12,
							weight: 700,
							italic: true,
							tracking: 0,
							hyphen: true,
						},
					],
				},
			],
		},
	],
};

describe('SpreadCanvas', () => {
	it('draws each line where Rust put it, in the font Rust used', () => {
		const { container } = render(
			<SpreadCanvas book={book} spread={spread} layout={layout} zoom={1} label="Page" />,
		);
		const frame = container.querySelector('[data-frame="t1"]')!;
		expect(frame).toHaveAttribute('transform', 'translate(10 10)');
		const words = frame.querySelectorAll('text');
		expect(words[0]).toHaveTextContent('Once upon a');
		expect(words[0]).toHaveAttribute('textLength', '50');
		expect(words[0]).toHaveAttribute('font-family', "'Literata', serif");
		expect(words[1]).toHaveAttribute('font-style', 'italic');
		expect(words[1]).toHaveAttribute('font-weight', '700');
		// The hyphen is drawn after the word and is not part of its text.
		expect(words[2]).toHaveTextContent('-');
		expect(words[2]).toHaveAttribute('x', '60');
	});

	it('draws the frames without their text until the layout arrives', () => {
		const { container } = render(
			<SpreadCanvas book={book} spread={spread} layout={null} zoom={1} label="Page" />,
		);
		expect(container.querySelector('[data-frame]')).toBeNull();
		expect(screen.getByRole('img', { name: 'Otter' })).toBeInTheDocument();
	});

	it('scales the page by the zoom, at 96 dpi', () => {
		render(<SpreadCanvas book={book} spread={spread} layout={null} zoom={2} label="Page" />);
		// 118 pt with its bleed, at 4/3 px per point and twice the zoom.
		expect(screen.getByRole('group', { name: 'Page' })).toHaveAttribute(
			'width',
			String(118 * (96 / 72) * 2),
		);
	});

	it('draws only the marks that are on', () => {
		const on = { ...DEFAULT_MARKS, columns: true, baseline: true, frameEdges: true };
		const { container, rerender } = render(
			<SpreadCanvas book={book} spread={spread} layout={null} zoom={1} marks={on} label="Page" />,
		);
		expect(container.querySelectorAll('[data-mark="columns"]')).toHaveLength(2);
		expect(container.querySelectorAll('[data-mark="baseline"]').length).toBeGreaterThan(3);
		expect(container.querySelectorAll('[data-mark="frame-edges"]')).toHaveLength(2);
		rerender(
			<SpreadCanvas
				book={book}
				spread={spread}
				layout={null}
				zoom={1}
				marks={{
					...on,
					bleed: false,
					margins: false,
					columns: false,
					baseline: false,
					frameEdges: false,
				}}
				label="Page"
			/>,
		);
		expect(container.querySelector('[data-mark]')).toBeNull();
	});
});
