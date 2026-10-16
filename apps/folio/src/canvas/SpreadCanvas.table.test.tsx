// Tests for tables on the canvas: drawing the cells' words, and editing them in place
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import type { TableEdit } from '@liminal-hq/folio-protocol/generated/TableEdit';
import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { PX_PER_POINT } from './geometry';
import { SpreadCanvas } from './SpreadCanvas';

const book = {
	trim: { w: 400, h: 400 },
	bleed: 0,
	margins: { top: 0, bottom: 0, inside: 0, outside: 0 },
	grid: { columns: 1, gutter: 0, baseline: null },
} as unknown as Book;

const table = (extra = {}) => ({
	type: 'table',
	id: 'tb',
	box: { x: 100, y: 100, w: 200, h: 60 },
	rotation: 0,
	locked: false,
	cols: [0.5, 0.5],
	rows: [
		[{ content: { inline: [{ text: 'Name' }] } }, {}],
		[{}, { content: { inline: [{ text: 'Pip' }] } }],
	],
	head: 1,
	look: 'lined',
	...extra,
});

const cellLayout = (frame: string, text: string, x: number, y: number) => ({
	frame,
	overset: false,
	textBox: { x, y, w: 92, h: 24 },
	lines: text
		? [
				{
					x,
					top: y,
					baseline: y + 10,
					width: 30,
					height: 12,
					runs: [
						{
							text,
							x: 0,
							width: 30,
							family: 'Inter',
							size: 12,
							weight: 400,
							italic: false,
							tracking: 0,
							hyphen: false,
						},
					],
				},
			]
		: [],
});

const layout = {
	spread: 's',
	preflight: [],
	frames: [
		cellLayout('tb#0.0', 'Name', 4, 3),
		cellLayout('tb#0.1', '', 104, 3),
		cellLayout('tb#1.0', '', 4, 33),
		cellLayout('tb#1.1', 'Pip', 104, 33),
	],
} as unknown as SpreadLayout;

function Harness({
	onEditTable,
	frame = table(),
}: {
	onEditTable?: (id: string, edit: TableEdit) => void;
	frame?: object;
}) {
	const [selection, setSelection] = useState<string[]>([]);
	const spread = {
		id: 's',
		kind: 'single',
		pages: [{ id: 'p', master: null }],
		frames: [frame],
		guides: [],
		masterOverrides: [],
	} as unknown as Spread;
	return (
		<SpreadCanvas
			book={book}
			spread={spread}
			layout={layout}
			zoom={1}
			label="Page"
			selection={selection}
			onSelect={setSelection}
			onEditText={vi.fn()}
			onEditTable={onEditTable}
		/>
	);
}

const px = (points: number) => points * PX_PER_POINT;
const stubBounds = () => {
	screen.getByRole('group', { name: 'Page' }).getBoundingClientRect = () =>
		({ left: 0, top: 0, width: 0, height: 0 }) as DOMRect;
};
const open = (x: number, y: number) => {
	stubBounds();
	fireEvent.doubleClick(document.querySelector('[data-hit="tb"]')!, {
		clientX: px(x),
		clientY: px(y),
	});
};

describe('a table on the canvas', () => {
	it('draws every cell’s words and the rules, and is named Table', () => {
		render(<Harness />);
		expect(document.querySelector('[data-table="tb"]')).not.toBeNull();
		expect(document.querySelector('[data-frame="tb#0.0"]')).not.toBeNull();
		expect(document.querySelector('[data-frame="tb#1.1"]')).not.toBeNull();
		expect(screen.getByRole('option', { name: 'Table' })).toBeInTheDocument();
	});

	it('edits the cell that was double-clicked, set in the cell’s own room', () => {
		render(<Harness onEditTable={vi.fn()} />);
		open(110, 150);
		const editor = screen.getByRole('textbox');
		expect(editor).toHaveValue('');
		// The cell is in the second row of the first column: the room starts 30 pt down and 4 in.
		expect(parseFloat(editor.style.left)).toBeCloseTo(px(100 + 4));
		expect(parseFloat(editor.style.top)).toBeCloseTo(px(100 + 33));
		expect(parseFloat(editor.style.width)).toBeCloseTo(px(92));
	});

	it('keeps the words typed as one change to that cell and shows the cell’s words again', async () => {
		const onEditTable = vi.fn();
		render(<Harness onEditTable={onEditTable} />);
		open(110, 110);
		expect(screen.getByRole('textbox')).toHaveValue('Name');
		await userEvent.type(screen.getByRole('textbox'), ' tag');
		fireEvent.keyDown(screen.getByRole('textbox'), { key: 'Escape' });
		expect(onEditTable).toHaveBeenCalledWith('tb', {
			type: 'setCell',
			row: 0,
			col: 0,
			text: 'Name tag',
		});
		expect(screen.queryByRole('textbox')).toBeNull();
	});

	it('sends nothing when nothing was typed', () => {
		const onEditTable = vi.fn();
		render(<Harness onEditTable={onEditTable} />);
		open(110, 110);
		fireEvent.keyDown(screen.getByRole('textbox'), { key: 'Escape' });
		expect(onEditTable).not.toHaveBeenCalled();
	});

	it('moves on with Tab, keeping the words, and back with Shift+Tab', async () => {
		const onEditTable = vi.fn();
		render(<Harness onEditTable={onEditTable} />);
		open(110, 110);
		await userEvent.keyboard('!{Tab}');
		expect(onEditTable).toHaveBeenCalledWith('tb', {
			type: 'setCell',
			row: 0,
			col: 0,
			text: 'Name!',
		});
		// Now in the next cell, along the row.
		const editor = screen.getByRole('textbox');
		expect(parseFloat(editor.style.left)).toBeCloseTo(px(100 + 104));
		await userEvent.keyboard('{Shift>}{Tab}{/Shift}');
		expect(parseFloat(screen.getByRole('textbox').style.left)).toBeCloseTo(px(100 + 4));
	});

	it('makes a new row when Tab leaves the last cell, and goes to its first cell', async () => {
		const onEditTable = vi.fn();
		render(<Harness onEditTable={onEditTable} />);
		open(250, 150);
		await userEvent.keyboard('{Tab}');
		expect(onEditTable).toHaveBeenCalledWith('tb', { type: 'addRow', at: 2 });
		// The row is not there until the book answers, so the editor waits for it.
		expect(screen.queryByRole('textbox')).toBeNull();
	});

	it('does not edit a locked table, or a cell with emphasis plain text would lose', () => {
		const { unmount } = render(<Harness onEditTable={vi.fn()} frame={table({ locked: true })} />);
		open(110, 110);
		expect(screen.queryByRole('textbox')).toBeNull();
		unmount();
		const bold = table({
			rows: [
				[{ content: { inline: [{ text: 'x', bold: true }] } }, {}],
				[{}, {}],
			],
		});
		render(<Harness onEditTable={vi.fn()} frame={bold} />);
		open(110, 110);
		expect(screen.queryByRole('textbox')).toBeNull();
	});
});
