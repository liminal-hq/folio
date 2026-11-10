// Tests the caret and a selection crossing from one frame of a story to the next on a spread
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { getTextCaret, getTextSelection } from '../canvas/textSelection';
import { CHAR_W, LINE_H, layoutOf } from './fixtureLayout';
import { TextSurface } from './TextSurface';

const lines = ['one two', 'three four', 'five six', 'seven eight'];
const words = lines.join(' ');
const all = layoutOf([lines]);
const slice = (from: number, to: number): FrameLayout => ({
	...all,
	lines: all.lines
		.slice(from, to)
		.map((l, i) => ({ ...l, top: i * LINE_H, baseline: i * LINE_H + 15 })),
});
// Frame A holds the first two lines; frame B, 300 points to the right, the last two.
const story = [
	{ id: 'a', box: { x: 0, y: 0, w: 200, h: 100 }, layout: slice(0, 2) },
	{ id: 'b', box: { x: 300, y: 0, w: 200, h: 100 }, layout: slice(2, 4) },
];
const startOf = (line: number) => all.lines[line]!.start;

function mount() {
	render(
		<TextSurface
			target={{ type: 'story', flow: 'fl' }}
			frameId="a"
			texts={[words]}
			layout={story[0]!.layout}
			story={story}
			box={{ w: 200, h: 100 }}
			at={{ x: 0, y: 0 }}
			scale={1}
			onEdit={async () => ({ caret: { para: 0, offset: 0 } })}
			onClose={() => undefined}
		/>,
	);
	return screen.getByRole('textbox') as HTMLTextAreaElement;
}

beforeEach(() => {
	vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({
		left: 0,
		top: 0,
		width: 200,
		height: 100,
		right: 200,
		bottom: 100,
		x: 0,
		y: 0,
		toJSON: () => ({}),
	});
});

const caretAt = () => getTextCaret()!.pos!;
const caretLeft = () =>
	Number.parseFloat((document.querySelector('[data-caret]') as HTMLElement).style.left);

describe('the caret in a story with two frames on the spread', () => {
	it('goes down from the last line of the first frame into the second, keeping its column', () => {
		const sink = mount();
		fireEvent.keyDown(sink, { key: 'ArrowDown' });
		fireEvent.keyDown(sink, { key: 'ArrowRight' });
		fireEvent.keyDown(sink, { key: 'ArrowRight' });
		fireEvent.keyDown(sink, { key: 'ArrowDown' });
		expect(caretAt()).toEqual({ para: 0, offset: startOf(2) + 2 });
		// It is drawn in the second frame, 300 points to the right.
		expect(caretLeft()).toBe(300 + 2 * CHAR_W);
		fireEvent.keyDown(sink, { key: 'ArrowUp' });
		expect(caretAt()).toEqual({ para: 0, offset: startOf(1) + 2 });
	});

	it('moves into the next frame with the arrow right across the break', () => {
		const sink = mount();
		fireEvent.keyDown(sink, { key: 'ArrowDown' });
		fireEvent.keyDown(sink, { key: 'ArrowRight' });
		fireEvent.keyDown(sink, { key: 'End' });
		expect(caretAt()).toEqual({ para: 0, offset: startOf(1) + 'three four'.length });
		fireEvent.keyDown(sink, { key: 'ArrowRight' });
		fireEvent.keyDown(sink, { key: 'ArrowRight' });
		expect(caretAt()).toEqual({ para: 0, offset: startOf(2) + 1 });
		expect(caretLeft()).toBe(300 + CHAR_W);
		// Home goes to the start of the line in the second frame.
		fireEvent.keyDown(sink, { key: 'Home' });
		expect(caretLeft()).toBe(300);
	});

	it('is put where the second frame is clicked, and a drag from the first reaches it', () => {
		const sink = mount();
		const sibling = document.querySelector('[data-sibling="b"]') as HTMLElement;
		fireEvent.mouseDown(sibling, { clientX: 300 + 3 * CHAR_W, clientY: LINE_H + 5 });
		expect(caretAt()).toEqual({ para: 0, offset: startOf(3) + 3 });
		// Drag from the first line of the first frame into the second.
		fireEvent.mouseDown(sink.parentElement!, { clientX: 0, clientY: 5 });
		act(() => {
			fireEvent.mouseMove(window, { clientX: 300 + 4 * CHAR_W, clientY: 5 });
			fireEvent.mouseUp(window);
		});
		expect(getTextSelection()).toMatchObject({
			from: { para: 0, offset: 0 },
			to: { para: 0, offset: startOf(2) + 4 },
		});
		// The selection is drawn in both frames.
		const rects = [...document.querySelectorAll('[data-selection]')] as HTMLElement[];
		expect(rects.some((r) => Number.parseFloat(r.style.left) < 300)).toBe(true);
		expect(rects.some((r) => Number.parseFloat(r.style.left) >= 300)).toBe(true);
	});

	it('draws nothing extra for a story with one frame on the spread', () => {
		render(
			<TextSurface
				target={{ type: 'story', flow: 'fl' }}
				frameId="a"
				texts={[words]}
				layout={story[0]!.layout}
				story={[story[0]!]}
				box={{ w: 200, h: 100 }}
				at={{ x: 0, y: 0 }}
				scale={1}
				onEdit={async () => ({ caret: { para: 0, offset: 0 } })}
				onClose={() => undefined}
			/>,
		);
		expect(document.querySelector('[data-sibling]')).toBeNull();
	});
});
