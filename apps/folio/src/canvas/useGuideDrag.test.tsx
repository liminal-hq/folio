// Tests for pulling a guide out of a ruler, moving one, and dropping one back on a ruler
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { act, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { useGuideDrag } from './useGuideDrag';

/** A stage at (100, 80) with its page's trim origin at (140, 120) px, which is 1 px a point. */
function stage() {
	const room = document.createElement('div');
	room.getBoundingClientRect = () => ({ left: 100, top: 80 }) as DOMRect;
	const page = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
	page.setAttribute('role', 'group');
	// The page's box starts 9 pt of bleed before the trim origin.
	page.getBoundingClientRect = () => ({ left: 131, top: 111 }) as DOMRect;
	room.append(page);
	return { current: room };
}

const bounds = { x: -9, y: -9, w: 200, h: 200 };
const targets = () => ({ x: [100], y: [] as number[] });

function setup(snap = true) {
	const onAdd = vi.fn();
	const onMove = vi.fn();
	const onRemove = vi.fn();
	const view = renderHook(() =>
		useGuideDrag({
			stage: stage(),
			bounds,
			scale: 1,
			targets,
			fineStep: 5,
			snap,
			onAdd,
			onMove,
			onRemove,
		}),
	);
	return { view, onAdd, onMove, onRemove };
}

const pointer = (type: string, x: number, y: number, init: PointerEventInit = {}) =>
	window.dispatchEvent(
		Object.assign(new MouseEvent(type, { clientX: x, clientY: y, ...init }), { pointerId: 1 }),
	);

afterEach(() => vi.restoreAllMocks());

describe('useGuideDrag', () => {
	it('pulls a vertical guide out of the left ruler and adds it where it is let go', () => {
		const { view, onAdd } = setup();
		act(() => view.result.current.fromRuler('y')({ button: 0 }));
		act(() => pointer('pointermove', 240, 300));
		// 240 px is 100 pt from the origin at 140: it snaps to the target at 100.
		expect(view.result.current.state).toMatchObject({ axis: 'x', pos: 100, index: null });
		act(() => pointer('pointerup', 240, 300));
		expect(onAdd).toHaveBeenCalledWith('x', 100);
		expect(view.result.current.state).toBeNull();
	});

	it("pulls a horizontal guide out of the top ruler, onto the unit's fine step", () => {
		const { view, onAdd } = setup();
		act(() => view.result.current.fromRuler('x')({ button: 0 }));
		act(() => pointer('pointermove', 300, 224));
		act(() => pointer('pointerup', 300, 224));
		// 224 − 120 = 104 pt, and the nearest step of 5 is 105.
		expect(onAdd).toHaveBeenCalledWith('y', 105);
	});

	it('lets Alt place a guide freely and a switched-off snap leave it alone', () => {
		const free = setup();
		act(() => free.view.result.current.fromRuler('y')({ button: 0 }));
		act(() => pointer('pointermove', 243, 300, { altKey: true }));
		act(() => pointer('pointerup', 243, 300));
		expect(free.onAdd).toHaveBeenCalledWith('x', 103);
		const off = setup(false);
		act(() => off.view.result.current.fromRuler('y')({ button: 0 }));
		act(() => pointer('pointermove', 243, 300));
		act(() => pointer('pointerup', 243, 300));
		expect(off.onAdd).toHaveBeenCalledWith('x', 103);
	});

	it('moves a guide, and takes it away when it is let go back over its ruler', () => {
		const { view, onMove, onRemove } = setup();
		act(() => view.result.current.fromGuide('x', 3)({ button: 0 }));
		act(() => pointer('pointermove', 200, 300));
		act(() => pointer('pointerup', 200, 300));
		expect(onMove).toHaveBeenCalledWith(3, 60);

		act(() => view.result.current.fromGuide('x', 3)({ button: 0 }));
		act(() => pointer('pointermove', 90, 300));
		expect(view.result.current.state).toMatchObject({ remove: true, index: 3 });
		act(() => pointer('pointerup', 90, 300));
		expect(onRemove).toHaveBeenCalledWith(3);
	});

	it('adds nothing when a new guide is dropped back on its ruler, or Escape is pressed', () => {
		const { view, onAdd } = setup();
		act(() => view.result.current.fromRuler('y')({ button: 0 }));
		act(() => pointer('pointermove', 90, 300));
		act(() => pointer('pointerup', 90, 300));
		act(() => view.result.current.fromRuler('y')({ button: 0 }));
		act(() => pointer('pointermove', 250, 300));
		act(() => window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' })));
		act(() => pointer('pointerup', 250, 300));
		expect(onAdd).not.toHaveBeenCalled();
		expect(view.result.current.state).toBeNull();
	});

	it('ignores a press that is not the main button', () => {
		const { view } = setup();
		act(() => view.result.current.fromRuler('y')({ button: 2 }));
		act(() => pointer('pointermove', 250, 300));
		expect(view.result.current.state).toBeNull();
	});
});
