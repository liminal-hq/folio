// Tests for opening a frame of a story: its words are edited as a whole, not in the frame
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { SpreadCanvas } from './SpreadCanvas';

const frame = (extra = {}) => ({
	type: 'text',
	id: 'f1',
	box: { x: 20, y: 20, w: 200, h: 100 },
	rotation: 0,
	locked: false,
	fromMaster: false,
	style: 'body',
	columns: 1,
	baselineLock: false,
	flow: { id: 'fl_1', index: 0 },
	...extra,
});

const spread = (frames: object[]) =>
	({
		id: 's',
		kind: 'single',
		pages: [{ id: 'p', master: null }],
		frames,
		guides: [],
		masterOverrides: [],
	}) as unknown as Spread;

const book = {
	trim: { w: 400, h: 400 },
	bleed: 0,
	margins: { top: 0, bottom: 0, inside: 0, outside: 0 },
	grid: { columns: 1, gutter: 0, baseline: null },
	masterPages: [{ id: 'm', name: 'A', frames: [], guides: [] }],
} as unknown as Book;

const open = (frames: object[], onEditStory = vi.fn(), onEditText = vi.fn()) => {
	render(
		<SpreadCanvas
			book={book}
			spread={spread(frames)}
			layout={{ spread: 's', preflight: [], frames: [] } as unknown as SpreadLayout}
			zoom={1}
			label="Page"
			selection={[]}
			onSelect={() => undefined}
			onEditText={onEditText}
			onEditStory={onEditStory}
		/>,
	);
	fireEvent.doubleClick(document.querySelector('[data-hit="f1"]')!);
	return { onEditStory, onEditText };
};

describe('opening a frame of a story', () => {
	it('opens the story, not a box over the frame', () => {
		const { onEditStory } = open([frame()]);
		expect(onEditStory).toHaveBeenCalledWith('fl_1');
		expect(screen.queryByRole('textbox', { name: 'Edit the words' })).toBeNull();
	});

	it('leaves a locked frame alone', () => {
		const { onEditStory } = open([frame({ locked: true })]);
		expect(onEditStory).not.toHaveBeenCalled();
	});

	it('still edits a frame with words of its own in place', () => {
		const { onEditStory } = open([frame({ flow: null })]);
		expect(onEditStory).not.toHaveBeenCalled();
		expect(screen.getByRole('textbox', { name: 'Edit the words' })).toBeInTheDocument();
	});
});
