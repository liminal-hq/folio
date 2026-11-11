// Tests for aligning and distributing frames
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import { describe, expect, it } from 'vitest';
import { alignMoves, distributeMoves } from './arrange';
import { turnedBounds } from './turned';

const frame = (id: string, x: number, y: number, w: number, h: number, rotation = 0) =>
	({ type: 'text', id, box: { x, y, w, h }, rotation, locked: false }) as unknown as Frame;

const pages = [
	{ x: 0, y: 0, w: 400, h: 600 },
	{ x: 400, y: 0, w: 400, h: 600 },
];

/** Applies moves to frames, as the book would. */
const after = (frames: Frame[], moves: ReturnType<typeof alignMoves>) =>
	frames.map((f) => ({ ...f, box: moves.find((m) => m.frame === f.id)?.box ?? f.box }));

describe('alignMoves', () => {
	const a = frame('a', 10, 20, 100, 50);
	const b = frame('b', 60, 200, 40, 80);

	it('lines several frames up with the box that holds them all', () => {
		expect(alignMoves([a, b], 'left', pages)).toEqual([
			{ frame: 'b', box: { x: 10, y: 200, w: 40, h: 80 } },
		]);
		expect(alignMoves([a, b], 'right', pages)).toEqual([
			{ frame: 'b', box: { x: 70, y: 200, w: 40, h: 80 } },
		]);
		expect(alignMoves([a, b], 'top', pages)).toEqual([
			{ frame: 'b', box: { x: 60, y: 20, w: 40, h: 80 } },
		]);
		expect(alignMoves([a, b], 'bottom', pages)).toEqual([
			{ frame: 'a', box: { x: 10, y: 230, w: 100, h: 50 } },
		]);
		const centred = after([a, b], alignMoves([a, b], 'centre', pages));
		expect(centred[0]!.box.x + 50).toBeCloseTo(centred[1]!.box.x + 20);
		const middled = after([a, b], alignMoves([a, b], 'middle', pages));
		expect(middled[0]!.box.y + 25).toBeCloseTo(middled[1]!.box.y + 40);
	});

	it('moves nothing that is already in line', () => {
		expect(alignMoves([a, frame('c', 10, 300, 20, 20)], 'left', pages)).toEqual([]);
	});

	it('lines a lone frame up with the page its middle is over', () => {
		expect(alignMoves([a], 'left', pages)[0]!.box.x).toBe(0);
		expect(alignMoves([a], 'right', pages)[0]!.box.x).toBe(300);
		expect(alignMoves([a], 'centre', pages)[0]!.box.x).toBe(150);
		expect(alignMoves([a], 'middle', pages)[0]!.box.y).toBe(275);
		expect(alignMoves([a], 'bottom', pages)[0]!.box.y).toBe(550);
		const onRight = frame('r', 500, 10, 60, 30);
		expect(alignMoves([onRight], 'left', pages)[0]!.box.x).toBe(400);
		expect(alignMoves([onRight], 'right', pages)[0]!.box.x).toBe(740);
		expect(alignMoves([a], 'left', [])).toEqual([]);
	});

	it('measures a turned frame on the box it fills', () => {
		// 100 × 20 turned a quarter fills 20 × 100 about the same middle.
		const upright = frame('u', 200, 100, 50, 50);
		const turned = frame('t', 20, 300, 100, 20, 90);
		const fills = turnedBounds(turned.box, 90);
		expect(fills).toMatchObject({ x: 60, w: 20 });
		const moved = after([upright, turned], alignMoves([upright, turned], 'left', pages));
		// The turned frame is already furthest left, so the upright one comes to its visible edge.
		expect(moved[0]!.box.x).toBe(60);
		expect(moved[1]!.box).toEqual(turned.box);
		const right = after([upright, turned], alignMoves([upright, turned], 'right', pages));
		expect(turnedBounds(right[1]!.box, 90).x + 20).toBeCloseTo(250);
		expect(right[1]!.box.x).toBe(turned.box.x + 170);
	});
});

describe('distributeMoves', () => {
	it('needs three frames', () => {
		expect(
			distributeMoves([frame('a', 0, 0, 10, 10), frame('b', 50, 0, 10, 10)], 'horizontal'),
		).toEqual([]);
	});

	it('leaves equal gaps and keeps the outer frames where they are', () => {
		const frames = [frame('a', 0, 0, 10, 10), frame('b', 20, 0, 30, 10), frame('c', 90, 0, 10, 10)];
		const moved = after(frames, distributeMoves(frames, 'horizontal'));
		// 100 wide, 50 of frames, two gaps of 25.
		expect(moved.map((f) => f.box.x)).toEqual([0, 35, 90]);
		const down = [frame('a', 0, 0, 10, 10), frame('b', 0, 10, 10, 10), frame('c', 0, 70, 10, 30)];
		expect(after(down, distributeMoves(down, 'vertical')).map((f) => f.box.y)).toEqual([0, 35, 70]);
	});

	it('sorts the frames by where they are, whatever order they were picked in', () => {
		const frames = [frame('c', 90, 0, 10, 10), frame('a', 0, 0, 10, 10), frame('b', 20, 0, 30, 10)];
		const moved = after(frames, distributeMoves(frames, 'horizontal'));
		expect(moved.map((f) => f.box.x)).toEqual([90, 0, 35]);
	});

	it('measures turned frames on the box they fill', () => {
		const a = frame('a', 0, 0, 10, 10);
		const b = frame('b', 20, 0, 40, 10, 90); // fills 10 × 40 about (40, 5): x 35..45
		const c = frame('c', 90, 0, 10, 10);
		const moved = after([a, b, c], distributeMoves([a, b, c], 'horizontal'));
		const fills = turnedBounds(moved[1]!.box, 90);
		// 100 wide, 30 of frames, gaps of 35: the turned frame's visible left edge is at 45.
		expect(fills.x).toBeCloseTo(45);
		expect(moved[1]!.box.w).toBe(40);
	});
});
