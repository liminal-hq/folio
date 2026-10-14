// Tests for the spread canvas: what it draws from a book and the lines Rust positioned
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { DEFAULT_MARKS } from './marksModel';
import { SpreadCanvas } from './SpreadCanvas';

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
		{
			type: 'image',
			id: 'i1',
			box: { x: 0, y: 50, w: 100, h: 50 },
			image: { asset: null, fit: 'fill', focus: [0.5, 0.5], crop: null },
			placeholderLabel: 'Otter',
		},
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

	it('draws every frame inside the page, so none of it lands in the page around it', () => {
		const { container } = render(
			<SpreadCanvas book={book} spread={spread} layout={layout} zoom={1} label="Page" />,
		);
		const svg = container.querySelector('svg')!;
		// A <text> outside an <svg> is just an unknown HTML element, drawn as a line of plain words.
		expect(container.querySelectorAll('text')).toHaveLength(svg.querySelectorAll('text').length);
		expect(container.querySelector('[data-frame="t1"]')!.closest('svg')).toBe(svg);
		expect(screen.getByRole('img', { name: 'Otter' }).closest('svg')).toBe(svg);
		expect(
			[...container.children[0]!.children].filter((e) => e !== svg && e.tagName === 'g'),
		).toEqual([]);
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

describe('pictures', () => {
	const pictureSpread = {
		id: 's2',
		kind: 'single',
		pages: [{ id: 'p', master: null }],
		frames: [
			{
				type: 'image',
				id: 'i1',
				box: { x: 10, y: 10, w: 80, h: 60 },
				image: { asset: 'assets/images/a.png', fit: 'fit', focus: [0.5, 0.5], crop: null },
				placeholderLabel: 'Otter',
			},
		],
		guides: [],
		masterOverrides: [],
	} as unknown as Spread;

	it('draws the picture once it has been read, fitted as asked', () => {
		const { container } = render(
			<SpreadCanvas
				book={book}
				spread={pictureSpread}
				layout={null}
				zoom={1}
				label="Page"
				images={{ 'assets/images/a.png': 'blob:abc' }}
			/>,
		);
		const image = container.querySelector('image')!;
		expect(image).toHaveAttribute('href', 'blob:abc');
		expect(image).toHaveAttribute('preserveAspectRatio', 'xMidYMid meet');
	});

	it('keeps the placeholder until the picture has been read', () => {
		const { container } = render(
			<SpreadCanvas book={book} spread={pictureSpread} layout={null} zoom={1} label="Page" />,
		);
		expect(container.querySelector('image')).toBeNull();
		expect(screen.getByRole('img', { name: 'Otter' })).toBeInTheDocument();
	});
});

describe('master pages', () => {
	it('draws the lines of a master frame where its origin puts them', () => {
		const withMaster: SpreadLayout = {
			spread: 's1',
			preflight: [],
			frames: [
				{
					frame: 'fr_pn@pg1',
					overset: false,
					origin: { x: 100, y: 300 },
					lines: [
						{
							x: 0,
							top: 0,
							baseline: 10,
							width: 10,
							height: 12,
							runs: [
								{
									text: '7',
									x: 0,
									width: 8,
									family: 'Literata',
									size: 10,
									weight: 400,
									italic: false,
									tracking: 0,
									hyphen: false,
								},
							],
						},
					],
				},
			],
		};
		const { container } = render(
			<SpreadCanvas book={book} spread={spread} layout={withMaster} zoom={1} label="Page" />,
		);
		const group = container.querySelector('[data-frame="fr_pn@pg1"]')!;
		expect(group).toHaveAttribute('transform', 'translate(100 300)');
		expect(group).toHaveTextContent('7');
	});

	it('draws a master shape on its page, behind the frames and out of the way of the pointer', () => {
		const withShape = {
			...book,
			masterPages: [
				{
					id: 'mst_a',
					name: 'A',
					frames: [
						{
							type: 'shape',
							id: 'rule',
							box: { x: 10, y: 90, w: 80, h: 2 },
							shape: 'rect',
							fill: '#336699',
						},
					],
				},
			],
		} as unknown as Book;
		const onMaster = {
			...spread,
			pages: [{ id: 'p1', master: 'mst_a' }],
		} as unknown as Spread;
		const { container } = render(
			<SpreadCanvas book={withShape} spread={onMaster} layout={null} zoom={1} label="Page" />,
		);
		const item = container.querySelector('[data-master-item="rule@p1"]')!;
		expect(item).toHaveAttribute('pointer-events', 'none');
		expect(item.querySelector('rect')).toHaveAttribute('fill', '#336699');
	});

	describe('ruler guides', () => {
		const guided = {
			...spread,
			guides: [
				{ axis: 'x', pos: 30, locked: false },
				{ axis: 'y', pos: 60, locked: true },
			],
		} as unknown as Spread;
		const canvas = (extra = {}) => (
			<SpreadCanvas
				book={book}
				spread={guided}
				layout={null}
				zoom={1}
				label="Page"
				onSelect={() => {}}
				{...extra}
			/>
		);

		it('draws a vertical guide at its x and a horizontal one at its y, locked ones dashed', () => {
			const { container } = render(canvas());
			const [vertical, horizontal] = [...container.querySelectorAll('[data-guide]')].map((g) =>
				g.querySelector('line')!,
			);
			expect(vertical).toHaveAttribute('x1', '30');
			expect(vertical).toHaveAttribute('x2', '30');
			expect(vertical).not.toHaveAttribute('stroke-dasharray');
			expect(horizontal).toHaveAttribute('y1', '60');
			expect(horizontal).toHaveAttribute('stroke-dasharray');
		});

		it("draws a master's guides on its page in the margin colour, out of the pointer's way", () => {
			const withMaster = {
				...book,
				masterPages: [
					{
						id: 'mst_a',
						name: 'A',
						frames: [],
						guides: [{ axis: 'x', pos: 30, locked: false }],
					},
				],
			} as unknown as Book;
			const onMaster = {
				...guided,
				pages: [{ id: 'p1', master: 'mst_a' }],
			} as unknown as Spread;
			const { container } = render(
				<SpreadCanvas book={withMaster} spread={onMaster} layout={null} zoom={1} label="Page" />,
			);
			const line = container.querySelector('[data-master-guide="mst_a"]')!;
			expect(line).toHaveAttribute('x1', '30');
			expect(line).toHaveAttribute('pointer-events', 'none');
		});

		it('draws none when the guides mark is off', () => {
			const { container } = render(canvas({ marks: { ...DEFAULT_MARKS, guides: false } }));
			expect(container.querySelector('[data-guide]')).toBeNull();
		});

		it('starts a move from a press on a free guide, and not from a locked one', () => {
			const onGuidePointerDown = vi.fn();
			const { container } = render(canvas({ onGuidePointerDown }));
			fireEvent.pointerDown(container.querySelector('[data-guide-hit="0"]')!, { button: 0 });
			expect(onGuidePointerDown).toHaveBeenCalledWith('x', 0, expect.anything());
			fireEvent.pointerDown(container.querySelector('[data-guide-hit="1"]')!, { button: 0 });
			expect(onGuidePointerDown).toHaveBeenCalledTimes(1);
		});

		it("asks for a guide's menu with a right-click", () => {
			const onGuideMenu = vi.fn();
			const { container } = render(canvas({ onGuideMenu }));
			fireEvent.contextMenu(container.querySelector('[data-guide-hit="1"]')!, {
				clientX: 12,
				clientY: 34,
			});
			expect(onGuideMenu).toHaveBeenCalledWith(1, { x: 12, y: 34 });
		});

		it('draws a guide being dragged where it would land, and hides the one it moves', () => {
			const drag = {
				axis: 'x',
				pos: 45,
				index: 0,
				remove: false,
				pointer: { x: 0, y: 0 },
			} as const;
			const { container, rerender } = render(canvas({ guideDrag: drag }));
			expect(container.querySelector('[data-guide="0"]')).toBeNull();
			expect(container.querySelector('[data-guide-drag="x"]')).toHaveAttribute('x1', '45');
			rerender(canvas({ guideDrag: { ...drag, remove: true } }));
			expect(container.querySelector('[data-guide-drag="x"]')).toHaveAttribute(
				'stroke',
				'var(--folio-danger)',
			);
		});
	});
});
