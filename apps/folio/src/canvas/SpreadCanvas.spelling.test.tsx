// Tests for the underlines the canvas draws under misspelt words
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { SpreadCanvas, squiggle } from './SpreadCanvas';

const book = {
	trim: { w: 400, h: 400 },
	bleed: 0,
	margins: { top: 0, bottom: 0, inside: 0, outside: 0 },
	grid: { columns: 1, gutter: 0, baseline: null },
} as unknown as Book;

const spread = {
	id: 's',
	kind: 'single',
	pages: [{ id: 'p', master: null }],
	frames: [
		{
			type: 'text',
			id: 'a',
			box: { x: 20, y: 20, w: 300, h: 100 },
			rotation: 0,
			locked: false,
			style: 'body',
			columns: 1,
		},
	],
	guides: [],
	masterOverrides: [],
} as unknown as Spread;

const run = (text: string, x: number, miss?: [number, number][]) => ({
	text,
	x,
	width: text.length * 6,
	family: 'Inter',
	size: 12,
	weight: 400,
	italic: false,
	tracking: 0,
	hyphen: false,
	...(miss ? { miss } : {}),
});

const layout = {
	spread: 's',
	preflight: [],
	frames: [
		{
			frame: 'a',
			overset: false,
			lines: [
				{
					x: 0,
					top: 0,
					baseline: 12,
					width: 100,
					height: 14,
					runs: [run('A', 0), run('ottr', 12, [[0, 4]]), run('Zorblat', 50, [[0, 3]])],
				},
			],
		},
	],
} as unknown as SpreadLayout;

describe('misspelt words on the canvas', () => {
	it('draws a wavy line under the marked part of a word only', () => {
		render(<SpreadCanvas book={book} spread={spread} layout={layout} zoom={1} label="Page" />);
		const marks = [...document.querySelectorAll('[data-miss-mark]')];
		expect(marks).toHaveLength(2);
		// The whole of "ottr" (4 letters over 24 points, from 12), and the first 3 of 7 letters of
		// "Zorblat" (over 42 points, from 50).
		expect(marks[0]!.getAttribute('d')!.startsWith('M12 ')).toBe(true);
		expect(marks[0]!.getAttribute('d')).toContain('L36 ');
		expect(marks[1]!.getAttribute('d')!.startsWith('M50 ')).toBe(true);
		expect(marks[1]!.getAttribute('d')).toContain('L68 ');
	});

	it('draws nothing under words that are right, or with no marks at all', () => {
		const plain = {
			...layout,
			frames: [
				{ ...layout.frames[0]!, lines: [{ ...layout.frames[0]!.lines[0]!, runs: [run('A', 0)] }] },
			],
		} as unknown as SpreadLayout;
		render(<SpreadCanvas book={book} spread={spread} layout={plain} zoom={1} label="Page" />);
		expect(document.querySelector('[data-miss-mark]')).toBeNull();
	});
});

describe('squiggle', () => {
	it('goes up and down along the length and ends where it should', () => {
		expect(squiggle(0, 6, 2, 1.5)).toBe('M0 2L3 0.5L6 3.5');
		expect(squiggle(0, 4, 2, 1.5)).toBe('M0 2L3 0.5L4 3.5');
	});
});
