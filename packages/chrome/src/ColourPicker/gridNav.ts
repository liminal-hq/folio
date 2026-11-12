// Arrow-key movement across the picker's swatch sections, which are grids of different widths
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** One section of swatches: how many there are and how many sit in a row. */
export interface GridSection {
	count: number;
	cols: number;
}

export interface GridPos {
	section: number;
	index: number;
}

export type GridKey = 'ArrowLeft' | 'ArrowRight' | 'ArrowUp' | 'ArrowDown' | 'Home' | 'End';

const filled = (sections: readonly GridSection[]) =>
	sections.map((s, i) => [s, i] as const).filter(([s]) => s.count > 0);

/**
 * Where `key` moves from `from`, or `null` when it stays put. Left and right walk the swatches in
 * reading order across sections; up and down keep the column, stepping into the section above or
 * below at the nearest column.
 */
export function moveInGrid(
	sections: readonly GridSection[],
	from: GridPos,
	key: GridKey,
): GridPos | null {
	const live = filled(sections);
	if (live.length === 0) return null;
	const first = { section: live[0]![1], index: 0 };
	const last = { section: live[live.length - 1]![1], index: live[live.length - 1]![0].count - 1 };
	if (key === 'Home') return first;
	if (key === 'End') return last;
	const here = sections[from.section];
	if (!here) return first;
	const at = live.findIndex(([, i]) => i === from.section);
	if (key === 'ArrowRight' || key === 'ArrowLeft') {
		const step = key === 'ArrowRight' ? 1 : -1;
		const next = from.index + step;
		if (next >= 0 && next < here.count) return { section: from.section, index: next };
		const other = live[at + step];
		if (!other) return null;
		return { section: other[1], index: step > 0 ? 0 : other[0].count - 1 };
	}
	const column = from.index % here.cols;
	if (key === 'ArrowDown') {
		if (from.index + here.cols < here.count)
			return { section: from.section, index: from.index + here.cols };
		const other = live[at + 1];
		if (!other) return null;
		return { section: other[1], index: Math.min(column, other[0].count - 1) };
	}
	if (from.index - here.cols >= 0) return { section: from.section, index: from.index - here.cols };
	const other = live[at - 1];
	if (!other) return null;
	const lastRow = Math.floor((other[0].count - 1) / other[0].cols) * other[0].cols;
	return { section: other[1], index: Math.min(lastRow + column, other[0].count - 1) };
}
