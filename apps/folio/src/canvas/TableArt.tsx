// Draws a table frame: the tint of its rows and the rules between its cells
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { TableFrame } from '@liminal-hq/folio-protocol/generated/TableFrame';

const RULE = '#14110f';
const TINT = '#14110f';

/** The x of each column's left edge, then the right edge of the table, in points on the page. */
export function columnEdges(frame: TableFrame): number[] {
	const edges = [frame.box.x];
	for (const share of frame.cols) edges.push(edges[edges.length - 1]! + frame.box.w * share);
	return edges;
}

/**
 * Rules are hairlines that keep their width at any zoom. Lined puts one under every row, Grid
 * puts one round every cell, and Recipe tints every other row below the header and rules under
 * the header only. The words are drawn over this by the page.
 */
export function TableArt({ frame }: { frame: TableFrame }) {
	const { x, y, w, h } = frame.box;
	const rows = frame.rows.length;
	const rowH = h / rows;
	const xs = columnEdges(frame);
	const line = (x1: number, y1: number, x2: number, y2: number, key: string, width = 0.75) => (
		<line
			key={key}
			x1={x1}
			y1={y1}
			x2={x2}
			y2={y2}
			stroke={RULE}
			strokeOpacity={0.7}
			strokeWidth={width}
			vectorEffect="non-scaling-stroke"
		/>
	);
	const rules = [];
	if (frame.look === 'grid') {
		for (let r = 0; r <= rows; r++) rules.push(line(x, y + r * rowH, x + w, y + r * rowH, `r${r}`));
		for (let c = 0; c < xs.length; c++) rules.push(line(xs[c]!, y, xs[c]!, y + h, `c${c}`));
	} else if (frame.look === 'lined') {
		for (let r = 1; r <= rows; r++) rules.push(line(x, y + r * rowH, x + w, y + r * rowH, `r${r}`));
	} else if (frame.head > 0) {
		const at = y + frame.head * rowH;
		rules.push(line(x, at, x + w, at, 'head', 1.25));
	}
	return (
		<g data-table={frame.id}>
			{frame.look === 'recipe' &&
				frame.rows.map((_, r) =>
					r >= frame.head && (r - frame.head) % 2 === 1 ? (
						<rect
							key={r}
							x={x}
							y={y + r * rowH}
							width={w}
							height={rowH}
							fill={TINT}
							fillOpacity={0.07}
						/>
					) : null,
				)}
			{rules}
		</g>
	);
}
