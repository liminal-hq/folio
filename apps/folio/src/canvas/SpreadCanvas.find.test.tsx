// Tests that the words the find bar found are marked on the page, and the current one is selected in the editor
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { FindMatch } from '@liminal-hq/folio-protocol/generated/FindMatch';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { render } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { FindDisplay } from '../find/matches';
import { layoutOf } from '../text/fixtureLayout';
import { SpreadCanvas } from './SpreadCanvas';

const book = {
	trim: { w: 400, h: 400 },
	bleed: 0,
	margins: { top: 0, bottom: 0, inside: 0, outside: 0 },
	grid: { columns: 1, gutter: 0, baseline: null },
	flows: [
		{
			id: 'fl',
			frames: ['b', 'c'],
			autoAddPages: true,
			paragraphs: [
				{ style: null, inline: [{ text: 'one two', style: null, bold: false, italic: false }] },
				{ style: null, inline: [{ text: 'three four', style: null, bold: false, italic: false }] },
			],
		},
	],
} as unknown as Book;

const frame = (id: string, x: number, extra: object = {}) => ({
	type: 'text',
	id,
	box: { x, y: 20, w: 150, h: 100 },
	rotation: 0,
	locked: false,
	style: 'body',
	columns: 1,
	content: { inline: [{ text: 'hello world', style: null, bold: false, italic: false }] },
	...extra,
});

const spread = {
	id: 's',
	kind: 'single',
	pages: [{ id: 'p', master: null }],
	frames: [
		frame('a', 20),
		frame('lock', 200, { locked: true }),
		frame('b', 20, { flow: { id: 'fl', index: 0 }, content: null }),
		frame('c', 200, { flow: { id: 'fl', index: 1 }, content: null }),
	],
	guides: [],
	masterOverrides: [],
} as unknown as Spread;

const layout = {
	spread: 's',
	preflight: [],
	frames: [
		{ ...layoutOf([['hello world']]), frame: 'a' },
		{ ...layoutOf([['hello world']]), frame: 'lock' },
		{ ...layoutOf([['one two']]), frame: 'b' },
		{
			...(layoutOf([['three four']]).lines.length
				? {
						lines: layoutOf([['three four']]).lines.map((l) => ({ ...l, paragraph: 1 })),
						overset: false,
					}
				: {}),
			frame: 'c',
		},
	],
} as never;

const found = (over: Partial<FindMatch>): FindMatch => ({
	target: { type: 'frame', frame: 'a' },
	para: 0,
	from: 6,
	to: 11,
	spread: 's',
	context: 'hello world',
	locked: false,
	...over,
});

function show(find: FindDisplay, newEditor = true) {
	const onEditRequest = vi.fn(async () => ({ caret: { para: 0, offset: 0 } }));
	const view = render(
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
			find={find}
		/>,
	);
	return view;
}

const marks = () => [...document.querySelectorAll('[data-find-match]')];

describe('the words found, marked on the page', () => {
	it('draws a box over each match, the current one stronger', () => {
		const matches = [found({}), found({ from: 0, to: 5 })];
		show({ matches, current: matches[0]!, focus: null });
		expect(marks()).toHaveLength(2);
		const current = document.querySelector('[data-find-match="current"]')!;
		// The frame is at 20; "world" starts six letters (10 points each) in.
		expect(current.getAttribute('x')).toBe('80');
		expect(current.getAttribute('width')).toBe('50');
		expect(document.querySelector('[data-find-match="other"]')!.getAttribute('x')).toBe('20');
	});

	it('marks words in a frame that cannot be edited, and in each frame of a story that shows them', () => {
		const matches = [
			found({ target: { type: 'frame', frame: 'lock' }, locked: true }),
			found({ target: { type: 'story', flow: 'fl' }, para: 0, from: 4, to: 7 }),
			found({ target: { type: 'story', flow: 'fl' }, para: 1, from: 6, to: 10 }),
		];
		show({ matches, current: null, focus: null });
		expect(marks()).toHaveLength(3);
		const xs = marks().map((m) => m.getAttribute('x'));
		expect(xs.filter((x) => x === '260')).toHaveLength(2);
		// "two" of the story's first paragraph is in the first frame; "four" of the second in the other.
		expect(xs).toContain('60');
	});

	it('draws nothing when nothing was found', () => {
		show({ matches: [], current: null, focus: null });
		expect(marks()).toHaveLength(0);
	});
});

describe('going to a match', () => {
	it('opens the editor on its frame with the words selected, and leaves the focus where it was', () => {
		const input = document.createElement('input');
		document.body.append(input);
		input.focus();
		const match = found({});
		show({ matches: [match], current: match, focus: { match, nonce: 1 } });
		expect(document.querySelector('[data-text-surface]')).not.toBeNull();
		const selected = document.querySelectorAll('[data-selection]');
		expect(selected).toHaveLength(1);
		expect(document.activeElement).toBe(input);
		input.remove();
	});

	it('selects words of a story in the frame that shows them', () => {
		const match = found({ target: { type: 'story', flow: 'fl' }, para: 1, from: 6, to: 10 });
		show({ matches: [match], current: match, focus: { match, nonce: 1 } });
		expect(document.querySelector('[data-text-surface]')).not.toBeNull();
		expect(document.querySelectorAll('[data-selection]').length).toBeGreaterThan(0);
	});

	it('only marks a locked frame, and does not open an editor on it', () => {
		const match = found({ target: { type: 'frame', frame: 'lock' }, locked: true });
		show({ matches: [match], current: match, focus: { match, nonce: 1 } });
		expect(document.querySelector('[data-text-surface]')).toBeNull();
		expect(marks()).toHaveLength(1);
	});

	it('only marks the match when the text box editor is in use', () => {
		const match = found({});
		show({ matches: [match], current: match, focus: { match, nonce: 1 } }, false);
		expect(document.querySelector('[data-text-surface]')).toBeNull();
		expect(marks()).toHaveLength(1);
	});

	it('goes again for a new request, to a different frame', () => {
		const first = found({});
		const view = show({ matches: [first], current: first, focus: { match: first, nonce: 1 } });
		const second = found({ target: { type: 'story', flow: 'fl' }, para: 1, from: 6, to: 10 });
		view.rerender(
			<SpreadCanvas
				book={book}
				spread={spread}
				layout={layout}
				zoom={1}
				label="Page"
				selection={[]}
				onSelect={() => undefined}
				onEditText={() => undefined}
				newEditor
				onEditRequest={async () => ({ caret: { para: 0, offset: 0 } })}
				find={{ matches: [second], current: second, focus: { match: second, nonce: 2 } }}
			/>,
		);
		const surface = document.querySelector('[data-text-surface]')!;
		expect(surface).not.toBeNull();
		expect(document.querySelectorAll('[data-selection]').length).toBeGreaterThan(0);
	});
});
