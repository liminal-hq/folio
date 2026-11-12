// Tests for arrow-key movement across swatch sections
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { moveInGrid, type GridSection } from './gridNav';

// A list of 3 (one per row), a pair, a grid of 10 in rows of 8, and an empty Recent row.
const sections: GridSection[] = [
	{ count: 3, cols: 1 },
	{ count: 2, cols: 2 },
	{ count: 10, cols: 8 },
	{ count: 0, cols: 8 },
];

describe('moveInGrid', () => {
	it('walks left and right in reading order across sections', () => {
		expect(moveInGrid(sections, { section: 0, index: 2 }, 'ArrowRight')).toEqual({
			section: 1,
			index: 0,
		});
		expect(moveInGrid(sections, { section: 2, index: 0 }, 'ArrowLeft')).toEqual({
			section: 1,
			index: 1,
		});
		expect(moveInGrid(sections, { section: 2, index: 9 }, 'ArrowRight')).toBeNull();
		expect(moveInGrid(sections, { section: 0, index: 0 }, 'ArrowLeft')).toBeNull();
	});

	it('keeps the column going down, stepping into the next section at the nearest column', () => {
		expect(moveInGrid(sections, { section: 1, index: 1 }, 'ArrowDown')).toEqual({
			section: 2,
			index: 1,
		});
		expect(moveInGrid(sections, { section: 2, index: 1 }, 'ArrowDown')).toEqual({
			section: 2,
			index: 9,
		});
		expect(moveInGrid(sections, { section: 2, index: 9 }, 'ArrowDown')).toBeNull();
	});

	it('goes up into the last row of the section above, and skips an empty section', () => {
		expect(moveInGrid(sections, { section: 2, index: 1 }, 'ArrowUp')).toEqual({
			section: 1,
			index: 1,
		});
		expect(moveInGrid(sections, { section: 2, index: 9 }, 'ArrowUp')).toEqual({
			section: 2,
			index: 1,
		});
		expect(moveInGrid(sections, { section: 0, index: 0 }, 'ArrowUp')).toBeNull();
	});

	it('jumps to the ends', () => {
		expect(moveInGrid(sections, { section: 1, index: 0 }, 'Home')).toEqual({
			section: 0,
			index: 0,
		});
		expect(moveInGrid(sections, { section: 1, index: 0 }, 'End')).toEqual({ section: 2, index: 9 });
	});
});
