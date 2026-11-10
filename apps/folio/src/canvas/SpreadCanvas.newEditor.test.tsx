// Tests that a text frame opens the on-page editor when the setting is on, and the text box when it is off
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { layoutOf } from '../text/fixtureLayout';
import { PX_PER_POINT } from './geometry';
import { SpreadCanvas } from './SpreadCanvas';

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
			content: { inline: [{ text: 'hello world', style: null, bold: false, italic: false }] },
		},
	],
	guides: [],
	masterOverrides: [],
} as unknown as Spread;

const layout = {
	spread: 's',
	preflight: [],
	frames: [layoutOf([['hello world']]).lines].map((lines) => ({
		frame: 'a',
		overset: false,
		lines,
	})),
} as never;

function open(newEditor: boolean) {
	const onEditRequest = vi.fn(async () => ({ caret: { para: 0, offset: 0 } }));
	render(
		<SpreadCanvas
			book={book}
			spread={spread}
			layout={layout}
			zoom={1}
			label="Page"
			selection={[]}
			onSelect={() => undefined}
			onEditText={() => undefined}
			newEditor={newEditor}
			onEditRequest={onEditRequest}
		/>,
	);
	fireEvent.doubleClick(document.querySelector('[data-hit="a"]')!);
	return onEditRequest;
}

describe('editing a text frame in place', () => {
	it('uses the on-page editor, and keeps drawing the frame lines under it, when the setting is on', () => {
		open(true);
		expect(document.querySelector('[data-text-surface]')).not.toBeNull();
		expect(document.querySelector('[data-frame="a"]')).not.toBeNull();
	});

	it('uses the text box when the setting is off', () => {
		open(false);
		expect(document.querySelector('[data-text-surface]')).toBeNull();
		expect(document.querySelector('textarea')).not.toBeNull();
	});
});

describe('editing the words of a story and of a table cell in place', () => {
	const storyBook = {
		...(book as object),
		flows: [
			{
				id: 'fl',
				frames: ['a'],
				autoAddPages: true,
				paragraphs: [
					{ style: null, inline: [{ text: 'one', style: null, bold: false, italic: false }] },
					{ style: null, inline: [{ text: 'two', style: null, bold: false, italic: false }] },
				],
			},
		],
	} as unknown as Book;

	it('opens the story over the frame it was opened in, as the words of the whole story', () => {
		const flowed = {
			...(spread as object),
			frames: [{ ...(spread.frames[0] as object), flow: { id: 'fl', index: 0 }, content: null }],
		} as unknown as Spread;
		const lines = layoutOf([['one'], ['two']]).lines;
		render(
			<SpreadCanvas
				book={storyBook}
				spread={flowed}
				layout={
					{ spread: 's', preflight: [], frames: [{ frame: 'a', overset: false, lines }] } as never
				}
				zoom={1}
				label="Page"
				selection={[]}
				onSelect={() => undefined}
				onEditText={() => undefined}
				newEditor
				onEditRequest={vi.fn(async () => ({ caret: { para: 0, offset: 0 } }))}
			/>,
		);
		fireEvent.doubleClick(document.querySelector('[data-hit="a"]')!);
		expect(document.querySelector('[data-text-surface]')).not.toBeNull();
		expect(document.querySelector('textarea')).not.toBeNull();
	});

	it('opens a table cell, and Tab moves to the next one', () => {
		const tableSpread = {
			...(spread as object),
			frames: [
				{
					type: 'table',
					id: 'tb',
					box: { x: 100, y: 100, w: 200, h: 60 },
					rotation: 0,
					locked: false,
					cols: [0.5, 0.5],
					rows: [[{ content: { inline: [{ text: 'Name' }] } }, {}]],
					head: 0,
					look: 'lined',
				},
			],
		} as unknown as Spread;
		const frames = ['tb#0.0', 'tb#0.1'].map((frame) => ({
			frame,
			overset: false,
			lines: layoutOf([['Name']]).lines,
		}));
		render(
			<SpreadCanvas
				book={book}
				spread={tableSpread}
				layout={{ spread: 's', preflight: [], frames } as never}
				zoom={1}
				label="Page"
				selection={[]}
				onSelect={() => undefined}
				onEditTable={() => undefined}
				newEditor
				onEditRequest={vi.fn(async () => ({ caret: { para: 0, offset: 0 } }))}
			/>,
		);
		screen.getByRole('group', { name: 'Page' }).getBoundingClientRect = () =>
			({ left: 0, top: 0, width: 0, height: 0 }) as DOMRect;
		fireEvent.doubleClick(document.querySelector('[data-hit="tb"]')!, {
			clientX: 110 * PX_PER_POINT,
			clientY: 110 * PX_PER_POINT,
		});
		const sink = document.querySelector('textarea')!;
		expect(document.querySelector('[data-text-surface]')).not.toBeNull();
		fireEvent.keyDown(sink, { key: 'Tab' });
		expect(document.querySelector('[data-text-surface]')).not.toBeNull();
	});
});
