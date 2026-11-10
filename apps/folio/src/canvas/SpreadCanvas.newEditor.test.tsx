// Tests that a text frame opens the on-page editor when the setting is on, and the text box when it is off
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { fireEvent, render } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { layoutOf } from '../text/fixtureLayout';
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
