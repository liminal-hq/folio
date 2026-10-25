// Tests for the wheel on the stage: Ctrl zooms, Shift moves across, and a plain wheel is left to scroll
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { act, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Stage } from './Stage';

const book = {
	trim: { w: 100, h: 100 },
	bleed: 9,
	margins: { top: 10, bottom: 10, inside: 10, outside: 10 },
	grid: { columns: 1, gutter: 6, baseline: { increment: 12, offset: 0 } },
} as unknown as Book;

const spread = {
	id: 's1',
	kind: 'opener',
	pages: [{ id: 'p1', master: null }],
	frames: [],
	guides: [],
	masterOverrides: [],
} as unknown as Spread;

const table = (container: HTMLElement) =>
	container.querySelector<HTMLElement>('[data-stage="table"]')!;

/** A wheel turn with the keys held; the test environment's own wheel events drop the modifiers. */
const turn = (
	target: HTMLElement,
	init: { deltaY?: number; deltaX?: number; ctrlKey?: boolean; shiftKey?: boolean },
): boolean => {
	const event = new Event('wheel', { bubbles: true, cancelable: true });
	Object.defineProperties(event, {
		deltaX: { value: init.deltaX ?? 0 },
		deltaY: { value: init.deltaY ?? 0 },
		deltaMode: { value: 0 },
		ctrlKey: { value: init.ctrlKey ?? false },
		shiftKey: { value: init.shiftKey ?? false },
		metaKey: { value: false },
		clientX: { value: 100 },
		clientY: { value: 100 },
	});
	let allowed = true;
	act(() => {
		allowed = target.dispatchEvent(event);
	});
	return allowed;
};

const level = (container: HTMLElement) =>
	Number(container.querySelector('output')?.textContent?.replace('%', ''));

describe('the wheel on the stage', () => {
	it('zooms in and out with Ctrl and the wheel, and stops following the stage size', () => {
		const { container } = render(<Stage book={book} spread={spread} label="Cover" layout={null} />);
		const area = table(container);
		const start = level(container);
		const turned = turn(area, { deltaY: -100, ctrlKey: true });
		// The browser's own page zoom is cancelled.
		expect(turned).toBe(false);
		const more = level(container);
		expect(more).toBeGreaterThan(start);
		turn(area, { deltaY: 100, ctrlKey: true });
		turn(area, { deltaY: 100, ctrlKey: true });
		const less = level(container);
		expect(less).toBeLessThan(start);
		expect(screen.getByRole('button', { name: 'Fit' })).toHaveAttribute('aria-pressed', 'false');
	});

	it('leaves a plain wheel to scroll', () => {
		const { container } = render(<Stage book={book} spread={spread} label="Cover" layout={null} />);
		const turned = turn(table(container), { deltaY: 100 });
		expect(turned).toBe(true);
	});

	it('moves across the table with Shift and the wheel when it can scroll', () => {
		const { container } = render(<Stage book={book} spread={spread} label="Cover" layout={null} />);
		const area = table(container);
		Object.defineProperty(area, 'scrollWidth', { configurable: true, value: 2000 });
		Object.defineProperty(area, 'clientWidth', { configurable: true, value: 500 });
		area.scrollLeft = 0;
		const turned = turn(area, { deltaY: 120, shiftKey: true });
		expect(turned).toBe(false);
		expect(area.scrollLeft).toBe(120);
	});
});
