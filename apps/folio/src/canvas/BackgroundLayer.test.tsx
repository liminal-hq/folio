// Tests for the background layer: what each page shows, and where it is painted
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Background } from '@liminal-hq/folio-protocol/generated/Background';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { BackgroundLayer } from './BackgroundLayer';
import { gradientLine, paintRects } from './backgrounds';

const book = {
	trim: { w: 100, h: 200 },
	bleed: 9,
	margins: { top: 10, bottom: 10, inside: 10, outside: 10 },
	masterPages: [],
} as unknown as Book;

const colour = (c: string): Background => ({ type: 'colour', colour: c });

function spreadOf(pages: number, extra: Partial<Spread> = {}): Spread {
	return {
		id: 's1',
		kind: pages === 2 ? 'spread' : 'opener',
		pages: Array.from({ length: pages }, (_, i) => ({ id: `p${i}`, master: null })),
		frames: [],
		guides: [],
		masterOverrides: [],
		...extra,
	} as unknown as Spread;
}

function drawn(spread: Spread) {
	const { container } = render(
		<svg>
			<BackgroundLayer book={book} spread={spread} />
		</svg>,
	);
	return container;
}

describe('paintRects', () => {
	it('reaches into the bleed on the outside edges only', () => {
		expect(paintRects(book, spreadOf(2))).toEqual([
			{ x: -9, y: -9, w: 109, h: 218 },
			{ x: 100, y: -9, w: 109, h: 218 },
		]);
		expect(paintRects(book, spreadOf(1))).toEqual([{ x: -9, y: -9, w: 118, h: 218 }]);
	});
});

describe('gradientLine', () => {
	it('runs left to right at 90 degrees and top to bottom at 180, clockwise from up like CSS', () => {
		const rect = { x: 0, y: 0, w: 200, h: 100 };
		const across = gradientLine(rect, 90);
		expect([across.x1, across.x2]).toEqual([0, 200]);
		expect(across.y1).toBeCloseTo(50);
		const down = gradientLine(rect, 180);
		expect(down.x1).toBeCloseTo(100);
		expect(down.y1).toBeCloseTo(0);
		expect(down.y2).toBeCloseTo(100);
	});

	it('reaches the corners when the line is slanted', () => {
		const line = gradientLine({ x: 0, y: 0, w: 100, h: 100 }, 45);
		expect(Math.hypot(line.x2 - line.x1, line.y2 - line.y1)).toBeCloseTo(100 * Math.SQRT2);
	});
});

describe('BackgroundLayer', () => {
	it('draws nothing when no page has a background', () => {
		expect(drawn(spreadOf(2)).querySelectorAll('[data-background]')).toHaveLength(0);
	});

	it("paints each page's own colour, clipped to its page", () => {
		const layer = drawn(
			spreadOf(2, {
				pages: [
					{ id: 'p0', master: null, bg: colour('#111111') },
					{ id: 'p1', master: null, bg: colour('#222222') },
				],
			} as Partial<Spread>),
		);
		const pages = layer.querySelectorAll('[data-background="page"]');
		expect(pages).toHaveLength(2);
		const fills = [...pages].map((g) => g.querySelector(':scope > rect')?.getAttribute('fill'));
		expect(fills).toEqual(['#111111', '#222222']);
	});

	it('paints a span once over the whole spread for each page that takes it', () => {
		const layer = drawn(spreadOf(2, { bgSpan: colour('#333333') }));
		const spans = layer.querySelectorAll('[data-background="span"] > rect');
		expect(spans).toHaveLength(2);
		for (const rect of spans) {
			expect(rect.getAttribute('x')).toBe('-9');
			expect(rect.getAttribute('width')).toBe('218');
		}
	});

	it('paints a gradient with a fill that points at its own definition', () => {
		const layer = drawn(
			spreadOf(1, {
				pages: [
					{
						id: 'p0',
						master: null,
						bg: {
							type: 'gradient',
							angle: 90,
							stops: [
								{ at: 0, colour: '#000000' },
								{ at: 1, colour: '#ffffff' },
							],
						},
					},
				],
			} as Partial<Spread>),
		);
		const gradient = layer.querySelector('linearGradient');
		expect(gradient).not.toBeNull();
		expect(gradient!.querySelectorAll('stop')).toHaveLength(2);
		const fill = layer.querySelector('[data-background] > rect')?.getAttribute('fill');
		expect(fill).toBe(`url(#${gradient!.id})`);
		expect(gradient!.id).not.toMatch(/[:\s]/);
	});

	it('paints a texture as a pattern tile, tinted when asked', () => {
		const layer = drawn(
			spreadOf(1, {
				pages: [{ id: 'p0', master: null, bg: { type: 'texture', id: 'linen', tint: '#aabbcc' } }],
			} as Partial<Spread>),
		);
		const pattern = layer.querySelector('pattern');
		expect(pattern).not.toBeNull();
		expect(pattern!.querySelector('rect')?.getAttribute('fill')).toBe('#aabbcc');
		expect(layer.querySelector('[data-background] > rect')?.getAttribute('fill')).toBe(
			`url(#${pattern!.id})`,
		);
	});

	it('leaves the paper showing for a texture this build cannot draw, and for a picture not yet drawn', () => {
		for (const bg of [
			{ type: 'texture', id: 'from-the-future' },
			{ type: 'picture', asset: 'assets/images/a.jpg', fit: 'fill', focus: [0.5, 0.5] },
		]) {
			const layer = drawn(
				spreadOf(1, { pages: [{ id: 'p0', master: null, bg }] } as unknown as Partial<Spread>),
			);
			expect(layer.querySelectorAll('rect[fill]')).toHaveLength(0);
		}
	});
});
