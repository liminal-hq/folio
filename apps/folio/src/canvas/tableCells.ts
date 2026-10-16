// Which cell of a table is where, and what is written in it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { TableFrame } from '@liminal-hq/folio-protocol/generated/TableFrame';
import { turnPoint } from './turnedHandles';

export interface CellAt {
	frame: string;
	row: number;
	col: number;
}

/** The id the layout gives a cell: its table's id, `#`, then its row and column. */
export const cellId = ({ frame, row, col }: CellAt) => `${frame}#${row}.${col}`;

/** The cell an id names, or `null` when it is not a cell's id. */
export function parseCellId(id: string): CellAt | null {
	const match = /^(.+)#(\d+)\.(\d+)$/.exec(id);
	return match ? { frame: match[1]!, row: Number(match[2]), col: Number(match[3]) } : null;
}

/** The cell under a point on the page, allowing for the table's turn; `null` outside the table. */
export function cellAt(table: TableFrame, point: { x: number; y: number }): CellAt | null {
	const { x, y, w, h } = table.box;
	const local = turnPoint(point, { x: x + w / 2, y: y + h / 2 }, -table.rotation);
	if (local.x < x || local.x > x + w || local.y < y || local.y > y + h) return null;
	const row = Math.min(table.rows.length - 1, Math.floor(((local.y - y) / h) * table.rows.length));
	let col = table.cols.length - 1;
	let edge = x;
	for (let c = 0; c < table.cols.length; c++) {
		edge += w * table.cols[c]!;
		if (local.x < edge) {
			col = c;
			break;
		}
	}
	return { frame: table.id, row, col };
}

/** The cell after (or before) this one, running along each row and on to the next; `null` off the ends. */
export function nextCell(table: TableFrame, from: CellAt, back: boolean): CellAt | null {
	const cols = table.cols.length;
	const at = from.row * cols + from.col + (back ? -1 : 1);
	if (at < 0 || at >= table.rows.length * cols) return null;
	return { frame: from.frame, row: Math.floor(at / cols), col: at % cols };
}

/** A cell's words as plain text, or `null` when they carry emphasis that plain text would lose. */
export function cellText(table: TableFrame, row: number, col: number): string | null {
	const runs = table.rows[row]?.[col]?.content?.inline ?? [];
	if (runs.some((r) => r.bold || r.italic || r.style)) return null;
	return runs.map((r) => r.text).join('');
}
