// Tests for drawing a turned shape: its drawing, its click area and its outline turn together
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { fireEvent, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { PX_PER_POINT } from './geometry';
import { SpreadCanvas } from './SpreadCanvas';
import type { FrameBoxMove } from './useFrameDrag';

const px = (points: number) => points * PX_PER_POINT;

const book = {
	trim: { w: 400, h: 400 },
	bleed: 0,
	margins: { top: 0, bottom: 0, inside: 0, outside: 0 },
	grid: { columns: 1, gutter: 0, baseline: null },
} as unknown as Book;

const shape = (rotation: number) => ({
	type: 'shape',
	id: 'a',
	box: { x: 100, y: 100, w: 100, h: 50 },
	rotation,
	locked: false,
	shape: 'rect',
	fill: '#ffffff',
	style: null,
});

function Harness({
	rotation,
	onMove,
	onTurn,
}: {
	rotation: number;
	onMove?: (moves: FrameBoxMove[]) => Promise<void>;
	onTurn?: (id: string, degrees: number) => Promise<void>;
}) {
	const [selection, setSelection] = useState<string[]>([]);
	const spread = {
		id: 's',
		kind: 'single',
		pages: [{ id: 'p', master: null }],
		frames: [shape(rotation)],
		guides: [],
		masterOverrides: [],
	} as unknown as Spread;
	return (
		<SpreadCanvas
			book={book}
			spread={spread}
			layout={null}
			zoom={1}
			label="Page"
			selection={selection}
			onSelect={setSelection}
			onMove={onMove}
			onTurn={onTurn}
		/>
	);
}

describe('a turned shape', () => {
	it('is drawn, picked and outlined turned about the middle of its box', () => {
		render(<Harness rotation={90} />);
		const turn = 'rotate(90 150 125)';
		const hit = document.querySelector('[data-hit="a"]')!;
		expect(hit.getAttribute('transform')).toBe(turn);
		fireEvent.pointerDown(hit, { button: 0 });
		expect(document.querySelector('[data-selected="a"]')!.getAttribute('transform')).toBe(turn);
		expect(
			document.querySelector('g[transform="' + turn + '"] > rect[fill="#ffffff"]'),
		).not.toBeNull();
	});

	it('has its handles on its turned corners, and a turn handle out from its top', () => {
		render(<Harness rotation={90} />);
		fireEvent.pointerDown(document.querySelector('[data-hit="a"]')!, { button: 0 });
		expect(document.querySelectorAll('rect[data-handle]')).toHaveLength(8);
		// Box 100,100 100×50 turned a quarter about (150, 125): its east edge faces down.
		const east = document.querySelector('[data-handle="e"]')!;
		expect(Number(east.getAttribute('x')) + Number(east.getAttribute('width')) / 2).toBeCloseTo(
			150,
		);
		expect(Number(east.getAttribute('y')) + Number(east.getAttribute('height')) / 2).toBeCloseTo(
			175,
		);
		const turn = document.querySelector('[data-handle="turn"]')!;
		expect(Number(turn.getAttribute('cx'))).toBeGreaterThan(180);
		expect(Number(turn.getAttribute('cy'))).toBeCloseTo(125);
	});

	it('has them square to the page when upright', () => {
		render(<Harness rotation={0} />);
		fireEvent.pointerDown(document.querySelector('[data-hit="a"]')!, { button: 0 });
		expect(document.querySelectorAll('rect[data-handle]')).toHaveLength(8);
		expect(document.querySelector('[data-hit="a"]')!.getAttribute('transform')).toBeNull();
	});

	it('resizes along its own sides, keeping the opposite edge where it was', async () => {
		const onMove = vi.fn(async (_moves: FrameBoxMove[]) => {});
		render(<Harness rotation={90} onMove={onMove} />);
		const svg = screen.getByRole('group', { name: 'Page' });
		svg.getBoundingClientRect = () => ({ left: 0, top: 0, width: 0, height: 0 }) as DOMRect;
		fireEvent.pointerDown(document.querySelector('[data-hit="a"]')!, { button: 0 });
		const east = document.querySelector('[data-handle="e"]')!;
		fireEvent.pointerDown(east, { button: 0, clientX: px(150), clientY: px(175) });
		// Down the page is out along the frame's east edge, so it grows 20 wider.
		fireEvent.pointerMove(svg, { clientX: px(150), clientY: px(195) });
		fireEvent.pointerUp(svg);
		await vi.waitFor(() => expect(onMove).toHaveBeenCalled());
		const rect = onMove.mock.calls[0]![0][0]!.rect;
		expect(rect.w).toBeCloseTo(120);
		expect(rect.h).toBeCloseTo(50);
		// The west edge, at (150, 75) on the page, has not moved: the middle slid 10 down.
		expect(rect.x + rect.w / 2).toBeCloseTo(150);
		expect(rect.y + rect.h / 2).toBeCloseTo(135);
	});

	it('turns with the turn handle, snapping to 15° with Shift, and sends one turn', async () => {
		const onTurn = vi.fn(async () => {});
		render(<Harness rotation={0} onTurn={onTurn} />);
		const svg = screen.getByRole('group', { name: 'Page' });
		svg.getBoundingClientRect = () => ({ left: 0, top: 0, width: 0, height: 0 }) as DOMRect;
		fireEvent.pointerDown(document.querySelector('[data-hit="a"]')!, { button: 0 });
		const handle = document.querySelector('[data-handle="turn"]')!;
		fireEvent.pointerDown(handle, { button: 0, clientX: px(150), clientY: px(80) });
		// Pointing along the page's east from the middle (150, 125) is a quarter turn.
		fireEvent.pointerMove(svg, { clientX: px(250), clientY: px(125) });
		expect(document.querySelector('[data-hit="a"]')!.getAttribute('transform')).toBe(
			'rotate(90 150 125)',
		);
		// A little below the east, with Shift, lands on 105 rather than the raw angle.
		fireEvent.pointerMove(svg, { clientX: px(250), clientY: px(150), shiftKey: true });
		fireEvent.pointerUp(svg);
		await vi.waitFor(() => expect(onTurn).toHaveBeenCalledTimes(1));
		expect(onTurn).toHaveBeenCalledWith('a', 105);
	});
});

describe('a turned master shape', () => {
	it('is drawn turned on the page it shows on', () => {
		const masterBook = {
			...book,
			masterPages: [{ id: 'm', name: 'A', guides: [], frames: [{ ...shape(90), id: 'ms' }] }],
		} as unknown as Book;
		const spread = {
			id: 's',
			kind: 'single',
			pages: [{ id: 'p', master: 'm' }],
			frames: [],
			guides: [],
			masterOverrides: [],
		} as unknown as Spread;
		render(<SpreadCanvas book={masterBook} spread={spread} layout={null} zoom={1} label="Page" />);
		expect(document.querySelector('[data-master-item="ms@p"]')!.getAttribute('transform')).toBe(
			'rotate(90 150 125)',
		);
	});
});
