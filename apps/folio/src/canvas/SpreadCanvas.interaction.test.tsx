// Tests for selecting, moving, resizing and nudging frames on the canvas
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { fireEvent, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import type { PlainRect } from './geometry';
import { PX_PER_POINT } from './geometry';
import { SpreadCanvas } from './SpreadCanvas';

const book = {
	trim: { w: 400, h: 400 },
	bleed: 0,
	margins: { top: 0, bottom: 0, inside: 0, outside: 0 },
	grid: { columns: 1, gutter: 0, baseline: null },
} as unknown as Book;

const frame = (id: string, x: number, y: number, extra = {}) => ({
	type: 'shape',
	id,
	box: { x, y, w: 100, h: 50 },
	rotation: 0,
	locked: false,
	shape: 'rect',
	fill: null,
	style: null,
	...extra,
});

const spread = {
	id: 's',
	kind: 'single',
	pages: [{ id: 'p', master: null }],
	frames: [frame('a', 50, 50), frame('b', 200, 200), frame('c', 50, 300, { locked: true })],
	guides: [],
	masterOverrides: [],
} as unknown as Spread;

function Harness({
	onMove = async () => {},
	onCommand = () => {},
}: {
	onMove?: (moves: { id: string; rect: PlainRect }[]) => Promise<void>;
	onCommand?: (command: string, ids: string[]) => void;
}) {
	const [selection, setSelection] = useState<string[]>([]);
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
			onCommand={onCommand}
		/>
	);
}

const movesOf = (fn: { mock: { calls: unknown[][] } }, call = 0) =>
	fn.mock.calls[call]![0] as { id: string; rect: PlainRect }[];

const hit = (id: string) => document.querySelector(`[data-hit="${id}"]`)!;
const px = (points: number) => points * PX_PER_POINT;

function stubBounds() {
	const svg = screen.getByRole('group', { name: 'Page' });
	svg.getBoundingClientRect = () => ({ left: 0, top: 0, width: 0, height: 0 }) as DOMRect;
}

describe('selecting', () => {
	it('selects the frame that is pressed and nothing else', () => {
		render(<Harness />);
		fireEvent.pointerDown(hit('a'), { button: 0 });
		expect(hit('a')).toHaveAttribute('aria-selected', 'true');
		fireEvent.pointerDown(hit('b'), { button: 0 });
		expect(hit('a')).toHaveAttribute('aria-selected', 'false');
		expect(hit('b')).toHaveAttribute('aria-selected', 'true');
	});

	it('adds and takes away frames with Shift', () => {
		render(<Harness />);
		fireEvent.pointerDown(hit('a'), { button: 0 });
		fireEvent.pointerDown(hit('b'), { button: 0, shiftKey: true });
		expect(hit('a')).toHaveAttribute('aria-selected', 'true');
		expect(hit('b')).toHaveAttribute('aria-selected', 'true');
		fireEvent.pointerDown(hit('a'), { button: 0, shiftKey: true });
		expect(hit('a')).toHaveAttribute('aria-selected', 'false');
	});

	it('lets go on a press on the page, and on Escape', () => {
		render(<Harness />);
		const svg = screen.getByRole('group', { name: 'Page' });
		fireEvent.pointerDown(hit('a'), { button: 0 });
		fireEvent.pointerUp(svg);
		fireEvent.keyDown(svg, { key: 'Escape' });
		expect(hit('a')).toHaveAttribute('aria-selected', 'false');
		fireEvent.pointerDown(hit('b'), { button: 0 });
		fireEvent.pointerUp(svg);
		fireEvent.pointerDown(svg.querySelector('rect')!, { button: 0 });
		expect(hit('b')).toHaveAttribute('aria-selected', 'false');
	});

	it('leaves Escape for others when there is nothing to let go of', () => {
		render(<Harness />);
		const svg = screen.getByRole('group', { name: 'Page' });
		expect(fireEvent.keyDown(svg, { key: 'Escape' })).toBe(true);
		fireEvent.pointerDown(hit('a'), { button: 0 });
		fireEvent.pointerUp(svg);
		expect(fireEvent.keyDown(svg, { key: 'Escape' })).toBe(false);
	});

	it('shows resize handles for one unlocked frame only', () => {
		render(<Harness />);
		fireEvent.pointerDown(hit('a'), { button: 0 });
		expect(document.querySelectorAll('[data-handle]')).toHaveLength(8);
		fireEvent.pointerDown(hit('b'), { button: 0, shiftKey: true });
		expect(document.querySelectorAll('[data-handle]')).toHaveLength(0);
		fireEvent.pointerDown(hit('c'), { button: 0 });
		expect(document.querySelectorAll('[data-handle]')).toHaveLength(0);
	});
});

describe('moving', () => {
	it('commits one move for a drag, in points', async () => {
		const onMove = vi.fn(async () => {});
		render(<Harness onMove={onMove} />);
		stubBounds();
		const svg = screen.getByRole('group', { name: 'Page' });
		fireEvent.pointerDown(hit('a'), { button: 0, clientX: px(60), clientY: px(60) });
		fireEvent.pointerMove(svg, { clientX: px(73), clientY: px(77), altKey: true });
		fireEvent.pointerUp(svg);
		await vi.waitFor(() => expect(onMove).toHaveBeenCalledTimes(1));
		const { id, rect } = movesOf(onMove)[0]!;
		expect(id).toBe('a');
		expect(rect.x).toBeCloseTo(63);
		expect(rect.y).toBeCloseTo(67);
		expect(rect.w).toBe(100);
	});

	it('does not move on a click that stays put', () => {
		const onMove = vi.fn(async () => {});
		render(<Harness onMove={onMove} />);
		stubBounds();
		const svg = screen.getByRole('group', { name: 'Page' });
		fireEvent.pointerDown(hit('a'), { button: 0, clientX: 100, clientY: 100 });
		fireEvent.pointerMove(svg, { clientX: 101, clientY: 100 });
		fireEvent.pointerUp(svg);
		expect(onMove).not.toHaveBeenCalled();
	});

	it('snaps an edge to another frame and draws the line', async () => {
		const onMove = vi.fn(async () => {});
		render(<Harness onMove={onMove} />);
		stubBounds();
		const svg = screen.getByRole('group', { name: 'Page' });
		// Frame a's left edge is at 50; dragging b so its left edge lands near 52 snaps it to 50.
		fireEvent.pointerDown(hit('b'), { button: 0, clientX: px(210), clientY: px(210) });
		fireEvent.pointerMove(svg, { clientX: px(62), clientY: px(210) });
		expect(document.querySelector('[data-snap="x"]')).not.toBeNull();
		fireEvent.pointerUp(svg);
		await vi.waitFor(() => expect(onMove).toHaveBeenCalled());
		expect(movesOf(onMove)[0]!.rect.x).toBe(50);
	});

	it('never moves a locked frame', () => {
		const onMove = vi.fn(async () => {});
		render(<Harness onMove={onMove} />);
		stubBounds();
		const svg = screen.getByRole('group', { name: 'Page' });
		fireEvent.pointerDown(hit('c'), { button: 0, clientX: px(60), clientY: px(310) });
		fireEvent.pointerMove(svg, { clientX: px(160), clientY: px(310) });
		fireEvent.pointerUp(svg);
		fireEvent.keyDown(svg, { key: 'ArrowRight' });
		expect(onMove).not.toHaveBeenCalled();
	});

	it('nudges by one point, or ten with Shift, and deletes', () => {
		const onMove = vi.fn(async () => {});
		const onCommand = vi.fn();
		render(<Harness onMove={onMove} onCommand={onCommand} />);
		const svg = screen.getByRole('group', { name: 'Page' });
		fireEvent.pointerDown(hit('a'), { button: 0 });
		fireEvent.keyDown(svg, { key: 'ArrowRight' });
		expect(onMove).toHaveBeenLastCalledWith([{ id: 'a', rect: { x: 51, y: 50, w: 100, h: 50 } }]);
		fireEvent.keyDown(svg, { key: 'ArrowUp', shiftKey: true });
		// The second press builds on the first, which the book has not shown yet.
		expect(onMove).toHaveBeenLastCalledWith([{ id: 'a', rect: { x: 51, y: 40, w: 100, h: 50 } }]);
		fireEvent.keyDown(svg, { key: 'Delete' });
		expect(onCommand).toHaveBeenCalledWith('delete', ['a']);
		fireEvent.keyDown(svg, { key: 'd', ctrlKey: true });
		expect(onCommand).toHaveBeenLastCalledWith('duplicate', ['a']);
	});
});

describe('several frames and a lost pointer', () => {
	it('moves the whole selection as one change', async () => {
		const onMove = vi.fn(async () => {});
		render(<Harness onMove={onMove} />);
		stubBounds();
		const svg = screen.getByRole('group', { name: 'Page' });
		fireEvent.pointerDown(hit('a'), { button: 0 });
		fireEvent.pointerUp(svg);
		fireEvent.pointerDown(hit('b'), {
			button: 0,
			shiftKey: true,
			clientX: px(210),
			clientY: px(210),
		});
		fireEvent.pointerMove(svg, { clientX: px(230), clientY: px(210), altKey: true });
		fireEvent.pointerUp(svg);
		await vi.waitFor(() => expect(onMove).toHaveBeenCalledTimes(1));
		const moves = movesOf(onMove);
		expect(moves.map((m) => m.id).sort()).toEqual(['a', 'b']);
		expect(moves.find((m) => m.id === 'a')!.rect.x).toBeCloseTo(70);
	});

	it('nudges a selection of several frames as one change too', () => {
		const onMove = vi.fn(async () => {});
		render(<Harness onMove={onMove} />);
		const svg = screen.getByRole('group', { name: 'Page' });
		fireEvent.pointerDown(hit('a'), { button: 0 });
		fireEvent.pointerUp(svg);
		fireEvent.pointerDown(hit('b'), { button: 0, shiftKey: true });
		fireEvent.pointerUp(svg);
		fireEvent.keyDown(svg, { key: 'ArrowDown' });
		expect(onMove).toHaveBeenCalledTimes(1);
		expect(movesOf(onMove)).toHaveLength(2);
	});

	it('drops a drag when the pointer is taken away, sending nothing', () => {
		const onMove = vi.fn(async () => {});
		render(<Harness onMove={onMove} />);
		stubBounds();
		const svg = screen.getByRole('group', { name: 'Page' });
		fireEvent.pointerDown(hit('a'), { button: 0, clientX: px(60), clientY: px(60) });
		fireEvent.pointerMove(svg, { clientX: px(90), clientY: px(90), altKey: true });
		fireEvent.pointerCancel(svg);
		// A later move with no button down must not carry on dragging.
		fireEvent.pointerMove(svg, { clientX: px(150), clientY: px(150), altKey: true });
		fireEvent.pointerUp(svg);
		expect(onMove).not.toHaveBeenCalled();
	});
});

describe('resizing', () => {
	it('drags a corner handle to a new size', async () => {
		const onMove = vi.fn(async () => {});
		render(<Harness onMove={onMove} />);
		stubBounds();
		const svg = screen.getByRole('group', { name: 'Page' });
		fireEvent.pointerDown(hit('a'), { button: 0 });
		const se = document.querySelector('[data-handle="se"]')!;
		fireEvent.pointerDown(se, { button: 0, clientX: px(150), clientY: px(100) });
		fireEvent.pointerMove(svg, { clientX: px(180), clientY: px(120), altKey: true });
		fireEvent.pointerUp(svg);
		await vi.waitFor(() => expect(onMove).toHaveBeenCalled());
		const rect = movesOf(onMove)[0]!.rect;
		expect(rect.x).toBe(50);
		expect(rect.w).toBeCloseTo(130);
		expect(rect.h).toBeCloseTo(70);
	});
});

describe('the frame menu', () => {
	it('asks for the menu on the frame pressed with the right button, selecting it first', () => {
		const onFrameMenu = vi.fn();
		const Wrapper = () => {
			const [selection, setSelection] = useState<string[]>([]);
			return (
				<SpreadCanvas
					book={book}
					spread={spread}
					layout={null}
					zoom={1}
					label="Page"
					selection={selection}
					onSelect={setSelection}
					onFrameMenu={onFrameMenu}
				/>
			);
		};
		render(<Wrapper />);
		fireEvent.contextMenu(hit('b'), { clientX: 30, clientY: 40 });
		expect(hit('b')).toHaveAttribute('aria-selected', 'true');
		expect(onFrameMenu).toHaveBeenCalledWith(expect.objectContaining({ id: 'b' }), {
			x: 30,
			y: 40,
		});
	});
});
