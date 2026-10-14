// Moving and resizing frames on the canvas with the pointer, and nudging them with the keys
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { KeyboardEvent, PointerEvent, RefObject } from 'react';
import { useEffect, useRef, useState } from 'react';
import type { FrameCommand } from './frameMenu';
import type { PlainRect, SpreadGeometry } from './geometry';
import {
	resizeRect,
	snapAxis,
	snapMove,
	snapTargets,
	SNAP_THRESHOLD_PX,
	type Handle,
	type Snapped,
} from './snap';

/** A pointer must travel this far, in screen pixels, before a press becomes a drag. */
const DRAG_SLOP_PX = 3;

type Drag =
	| {
			kind: 'move';
			ids: string[];
			origin: Map<string, PlainRect>;
			start: Point;
			px: Point;
			moved: boolean;
	  }
	| {
			kind: 'resize';
			id: string;
			handle: Handle;
			origin: PlainRect;
			start: Point;
			px: Point;
			moved: boolean;
	  };

interface Point {
	x: number;
	y: number;
}

/** A frame and the box it is to have. */
export interface FrameBoxMove {
	id: string;
	rect: PlainRect;
}

export interface FrameDragOptions {
	svg: RefObject<SVGSVGElement | null>;
	geometry: SpreadGeometry;
	frames: Frame[];
	/** Screen pixels per point. */
	scale: number;
	selection: string[];
	/** Whether a drag snaps to edges, centres and margins. Alt turns it off for one drag. */
	snap?: boolean;
	/** Ruler guides a drag can snap to, when they are shown. */
	guides?: { axis: 'x' | 'y'; pos: number }[];
	onSelect?(ids: string[]): void;
	/** Commits the new boxes of one or more frames as one change; resolves once the book has them. */
	onMove?(moves: FrameBoxMove[]): Promise<void>;
	/** A command on the selection, from the keys: duplicate, delete. */
	onCommand?(command: FrameCommand, ids: string[]): void;
}

/** Whether a frame can be dragged: not locked, and not a master's. */
export function movable(frame: Frame): boolean {
	return !frame.locked && !frame.fromMaster;
}

const same = (a: PlainRect, b: PlainRect) =>
	a.x === b.x && a.y === b.y && a.w === b.w && a.h === b.h;

export function useFrameDrag(options: FrameDragOptions) {
	const {
		svg,
		geometry,
		frames,
		scale,
		selection,
		snap = true,
		guides = [],
		onSelect,
		onMove,
		onCommand,
	} = options;
	const drag = useRef<Drag | null>(null);
	/** Where each frame was last nudged to, until the book shows it there. */
	const nudged = useRef(new Map<string, PlainRect>());
	const [preview, setPreview] = useState<Record<string, PlainRect>>({});
	const [lines, setLines] = useState<Snapped['lines']>([]);

	// A new copy of the frames is the book answering, so what was asked for is no longer pending.
	useEffect(() => {
		nudged.current.clear();
	}, [frames]);

	const boxOf = (id: string): PlainRect | null => frames.find((f) => f.id === id)?.box ?? null;

	const toPoint = (event: { clientX: number; clientY: number }): Point => {
		const rect = svg.current?.getBoundingClientRect();
		const bounds = geometry.bounds;
		return {
			x: bounds.x + (event.clientX - (rect?.left ?? 0)) / scale,
			y: bounds.y + (event.clientY - (rect?.top ?? 0)) / scale,
		};
	};

	const begin = (event: PointerEvent, next: Drag) => {
		drag.current = next;
		(event.currentTarget as Element).setPointerCapture?.(event.pointerId);
	};

	const onFramePointerDown = (event: PointerEvent, frame: Frame) => {
		if (event.button !== 0) return;
		event.stopPropagation();
		let ids = selection;
		if (event.shiftKey) {
			ids = selection.includes(frame.id)
				? selection.filter((id) => id !== frame.id)
				: [...selection, frame.id];
			onSelect?.(ids);
		} else if (!selection.includes(frame.id)) {
			ids = [frame.id];
			onSelect?.(ids);
		}
		const draggable = ids.filter((id) => {
			const f = frames.find((x) => x.id === id);
			return f !== undefined && movable(f);
		});
		if (draggable.length === 0 || !ids.includes(frame.id)) return;
		const origin = new Map(draggable.map((id) => [id, { ...boxOf(id)! }]));
		begin(event, {
			kind: 'move',
			ids: draggable,
			origin,
			start: toPoint(event),
			px: { x: event.clientX, y: event.clientY },
			moved: false,
		});
	};

	const onHandlePointerDown = (event: PointerEvent, id: string, handle: Handle) => {
		if (event.button !== 0) return;
		event.stopPropagation();
		const origin = boxOf(id);
		if (!origin) return;
		begin(event, {
			kind: 'resize',
			id,
			handle,
			origin: { ...origin },
			start: toPoint(event),
			px: { x: event.clientX, y: event.clientY },
			moved: false,
		});
	};

	const onBackgroundPointerDown = (event: PointerEvent) => {
		if (event.button !== 0 || event.shiftKey) return;
		onSelect?.([]);
	};

	const onPointerMove = (event: PointerEvent) => {
		const d = drag.current;
		if (!d) return;
		if (!d.moved) {
			const far = Math.hypot(event.clientX - d.px.x, event.clientY - d.px.y) >= DRAG_SLOP_PX;
			if (!far) return;
			d.moved = true;
		}
		const now = toPoint(event);
		let dx = now.x - d.start.x;
		let dy = now.y - d.start.y;
		const threshold = SNAP_THRESHOLD_PX / scale;
		const targets = snapTargets(geometry, frames, d.kind === 'move' ? d.ids : [d.id], guides);
		if (d.kind === 'move') {
			if (event.shiftKey) {
				// Shift keeps the move on one axis: the one the pointer has gone further along.
				if (Math.abs(dx) >= Math.abs(dy)) dy = 0;
				else dx = 0;
			}
			const lead = d.origin.get(d.ids[0]!)!;
			let applied = { dx, dy };
			let drawn: Snapped['lines'] = [];
			if (snap && !event.altKey) {
				const snapped = snapMove({ ...lead, x: lead.x + dx, y: lead.y + dy }, targets, threshold);
				applied = { dx: snapped.rect.x - lead.x, dy: snapped.rect.y - lead.y };
				drawn = snapped.lines;
			}
			setLines(drawn);
			setPreview(
				Object.fromEntries(
					d.ids.map((id) => {
						const o = d.origin.get(id)!;
						return [id, { ...o, x: o.x + applied.dx, y: o.y + applied.dy }];
					}),
				),
			);
			return;
		}
		let next = resizeRect(d.origin, d.handle, dx, dy, event.shiftKey);
		const drawn: Snapped['lines'] = [];
		if (snap && !event.altKey) {
			// Snap the edges this handle is dragging, then resize again with the correction.
			const xs = [
				...(d.handle.includes('w') ? [next.x] : []),
				...(d.handle.includes('e') ? [next.x + next.w] : []),
			];
			const ys = [
				...(d.handle.includes('n') ? [next.y] : []),
				...(d.handle.includes('s') ? [next.y + next.h] : []),
			];
			const sx = snapAxis(xs, targets.x, threshold);
			const sy = snapAxis(ys, targets.y, threshold);
			if (sx || sy) {
				next = resizeRect(
					d.origin,
					d.handle,
					dx + (sx?.shift ?? 0),
					dy + (sy?.shift ?? 0),
					event.shiftKey,
				);
			}
			if (sx) drawn.push({ axis: 'x', pos: sx.target });
			if (sy) drawn.push({ axis: 'y', pos: sy.target });
		}
		setLines(drawn);
		setPreview({ [d.id]: next });
	};

	const finish = async (commit: boolean) => {
		const d = drag.current;
		drag.current = null;
		const boxes = preview;
		setLines([]);
		if (!d || !d.moved || !commit) {
			setPreview({});
			return;
		}
		const moves = Object.entries(boxes).flatMap(([id, rect]) => {
			const was = boxOf(id);
			return was && !same(was, rect) ? [{ id, rect }] : [];
		});
		try {
			if (moves.length > 0) await onMove?.(moves);
		} finally {
			setPreview({});
		}
	};

	const onPointerUp = () => void finish(true);

	/** The pointer was taken away (a touch or pen cancel, a dialog, Alt-Tab): the drag is dropped. */
	const onPointerCancel = () => {
		if (drag.current) void finish(false);
	};

	const onKeyDown = (event: KeyboardEvent) => {
		if (event.key === 'Escape') {
			if (drag.current) void finish(false);
			else onSelect?.([]);
			event.preventDefault();
			return;
		}
		if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'a') {
			onSelect?.(frames.map((f) => f.id));
			event.preventDefault();
			return;
		}
		if (selection.length === 0) return;
		if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'd') {
			onCommand?.('duplicate', selection);
			event.preventDefault();
			return;
		}
		if (event.key === 'Delete' || event.key === 'Backspace') {
			onCommand?.('delete', selection);
			event.preventDefault();
			return;
		}
		const step = event.shiftKey ? 10 : 1;
		const by: Record<string, [number, number]> = {
			ArrowLeft: [-step, 0],
			ArrowRight: [step, 0],
			ArrowUp: [0, -step],
			ArrowDown: [0, step],
		};
		const delta = by[event.key];
		if (!delta) return;
		event.preventDefault();
		// Each press builds on the last one asked for, since the book may not have answered it yet.
		const moves: FrameBoxMove[] = [];
		for (const id of selection) {
			const frame = frames.find((f) => f.id === id);
			if (!frame || !movable(frame)) continue;
			const from = nudged.current.get(id) ?? frame.box;
			const rect = { ...from, x: from.x + delta[0], y: from.y + delta[1] };
			nudged.current.set(id, rect);
			moves.push({ id, rect });
		}
		if (moves.length > 0) void onMove?.(moves);
	};

	return {
		preview,
		lines,
		onFramePointerDown,
		onHandlePointerDown,
		onBackgroundPointerDown,
		onPointerMove,
		onPointerUp,
		onPointerCancel,
		onKeyDown,
	};
}
