// Tests for drawing a turned shape: its drawing, its click area and its outline turn together
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { fireEvent, render } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { SpreadCanvas } from './SpreadCanvas';

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

function Harness({ rotation }: { rotation: number }) {
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

	it('has no resize handles while it is turned, and has them upright', () => {
		const { unmount } = render(<Harness rotation={90} />);
		fireEvent.pointerDown(document.querySelector('[data-hit="a"]')!, { button: 0 });
		expect(document.querySelectorAll('[data-handle]')).toHaveLength(0);
		unmount();
		render(<Harness rotation={0} />);
		fireEvent.pointerDown(document.querySelector('[data-hit="a"]')!, { button: 0 });
		expect(document.querySelectorAll('[data-handle]')).toHaveLength(8);
		expect(document.querySelector('[data-hit="a"]')!.getAttribute('transform')).toBeNull();
	});
});
