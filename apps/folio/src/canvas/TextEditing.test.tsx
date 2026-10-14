// Tests for editing a text frame's words in place on the canvas
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { fireEvent, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { SpreadCanvas } from './SpreadCanvas';

const book = {
	trim: { w: 400, h: 400 },
	bleed: 0,
	margins: { top: 0, bottom: 0, inside: 0, outside: 0 },
	grid: { columns: 1, gutter: 0, baseline: null },
} as unknown as Book;

const text = (id: string, inline: object[], extra = {}) => ({
	type: 'text',
	id,
	box: { x: 20, y: 20, w: 200, h: 60 },
	rotation: 0,
	locked: false,
	style: 'body',
	columns: 1,
	baselineLock: false,
	content: { inline },
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

function Harness({
	frames,
	onEditText,
}: {
	frames: object[];
	onEditText: (id: string, t: string) => void;
}) {
	const [selection, setSelection] = useState<string[]>([]);
	return (
		<SpreadCanvas
			book={book}
			spread={spread(frames)}
			layout={null}
			zoom={1}
			label="Page"
			selection={selection}
			onSelect={setSelection}
			onEditText={onEditText}
		/>
	);
}

const hit = (id: string) => document.querySelector(`[data-hit="${id}"]`)!;

describe('editing words in place', () => {
	const frames = [text('a', [{ text: 'Once upon a time' }])];

	it('opens on a double-click with the words in it, and keeps the change on Escape', () => {
		const onEditText = vi.fn();
		render(<Harness frames={frames} onEditText={onEditText} />);
		fireEvent.doubleClick(hit('a'));
		const area = screen.getByRole('textbox', { name: 'Edit the words' });
		expect(area).toHaveValue('Once upon a time');
		fireEvent.change(area, { target: { value: 'The otter slept' } });
		fireEvent.keyDown(area, { key: 'Escape' });
		expect(onEditText).toHaveBeenCalledWith('a', 'The otter slept');
		expect(screen.queryByRole('textbox')).toBeNull();
	});

	it('opens on Enter for the selected frame', () => {
		render(<Harness frames={frames} onEditText={() => {}} />);
		fireEvent.pointerDown(hit('a'), { button: 0 });
		fireEvent.keyDown(screen.getByRole('group', { name: 'Page' }), { key: 'Enter' });
		expect(screen.getByRole('textbox')).toBeInTheDocument();
	});

	it('sends nothing when the words were not changed', () => {
		const onEditText = vi.fn();
		render(<Harness frames={frames} onEditText={onEditText} />);
		fireEvent.doubleClick(hit('a'));
		fireEvent.keyDown(screen.getByRole('textbox'), { key: 'Escape' });
		expect(onEditText).not.toHaveBeenCalled();
	});

	it('keeps the change when the area loses focus', () => {
		const onEditText = vi.fn();
		render(<Harness frames={frames} onEditText={onEditText} />);
		fireEvent.doubleClick(hit('a'));
		const area = screen.getByRole('textbox');
		fireEvent.change(area, { target: { value: 'Changed' } });
		fireEvent.blur(area);
		expect(onEditText).toHaveBeenCalledWith('a', 'Changed');
	});

	it.each([
		['words with emphasis', [text('b', [{ text: 'Bold', bold: true }])]],
		['a locked frame', [text('b', [{ text: 'Plain' }], { locked: true })]],
		[
			'a frame whose words come from a flow',
			[text('b', [{ text: 'Plain' }], { flow: { id: 'f', index: 0 } })],
		],
	])('does not open on %s', (_, list) => {
		render(<Harness frames={list} onEditText={() => {}} />);
		fireEvent.doubleClick(hit('b'));
		expect(screen.queryByRole('textbox')).toBeNull();
	});
});

describe('editing when the frame goes away', () => {
	it('keeps what was typed when the editor is closed without a blur', () => {
		const onEditText = vi.fn();
		const { unmount } = render(
			<Harness frames={[text('a', [{ text: 'Once' }])]} onEditText={onEditText} />,
		);
		fireEvent.doubleClick(hit('a'));
		fireEvent.change(screen.getByRole('textbox'), { target: { value: 'Typed, then gone' } });
		unmount();
		expect(onEditText).toHaveBeenCalledWith('a', 'Typed, then gone');
	});
});
