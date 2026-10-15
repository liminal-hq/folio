// Tests for the words in shapes on the canvas: where they are drawn and how they are edited
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { SpreadCanvas } from './SpreadCanvas';

const shape = (id: string, extra = {}) => ({
	type: 'shape',
	id,
	box: { x: 20, y: 20, w: 200, h: 100 },
	rotation: 0,
	locked: false,
	shape: 'speech',
	fill: '#ffffff',
	style: null,
	...extra,
});

const lines = (frame: string, extra = {}) => ({
	frame,
	overset: false,
	lines: [
		{
			x: 30,
			top: 30,
			baseline: 42,
			width: 40,
			height: 14,
			runs: [
				{
					text: 'Hello',
					x: 0,
					width: 40,
					family: 'Inter',
					size: 12,
					weight: 400,
					italic: false,
					tracking: 0,
					hyphen: false,
				},
			],
		},
	],
	...extra,
});

const spreadOf = (frames: object[], master: string | null = null) =>
	({
		id: 's',
		kind: 'single',
		pages: [{ id: 'p', master }],
		frames,
		guides: [],
		masterOverrides: [],
	}) as unknown as Spread;

const book = (masterFrames: object[] = []) =>
	({
		trim: { w: 400, h: 400 },
		bleed: 0,
		margins: { top: 0, bottom: 0, inside: 0, outside: 0 },
		grid: { columns: 1, gutter: 0, baseline: null },
		masterPages: [{ id: 'm', name: 'A', frames: masterFrames, guides: [] }],
	}) as unknown as Book;

describe('words in a master shape', () => {
	it('are drawn over the shape, with it, and not behind in the master layer', () => {
		const layout = {
			spread: 's',
			preflight: [],
			frames: [lines('ms@p', { origin: { x: 20, y: 20 } })],
		} as unknown as SpreadLayout;
		render(
			<SpreadCanvas
				book={book([shape('ms')])}
				spread={spreadOf([], 'm')}
				layout={layout}
				zoom={1}
				label="Page"
			/>,
		);
		const item = document.querySelector('[data-master-item="ms@p"]')!;
		expect(item.querySelector('[data-frame="ms@p"]')).not.toBeNull();
		expect(document.querySelectorAll('[data-frame="ms@p"]')).toHaveLength(1);
	});
});

describe('editing the words of a shape', () => {
	const open = (textAlign?: string) => {
		const layout = {
			spread: 's',
			preflight: [],
			frames: [
				lines('b', {
					textBox: { x: 16, y: 8, w: 168, h: 54 },
					...(textAlign ? { textAlign } : {}),
				}),
			],
		} as unknown as SpreadLayout;
		render(
			<SpreadCanvas
				book={book()}
				spread={spreadOf([shape('b')])}
				layout={layout}
				zoom={1}
				label="Page"
				selection={[]}
				onSelect={() => undefined}
				onEditText={() => undefined}
			/>,
		);
		fireEvent.doubleClick(document.querySelector('[data-hit="b"]')!);
		return screen.getByRole('textbox', { name: 'Edit the words' });
	};

	it('sets the words as the style sets them: centred, or to the right', () => {
		expect(open('center').style.textAlign).toBe('center');
	});

	it('sets them to the right when the style does', () => {
		expect(open('right').style.textAlign).toBe('right');
	});
});
