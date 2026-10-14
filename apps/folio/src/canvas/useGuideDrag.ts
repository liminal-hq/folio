// Pulling a ruler guide out of a ruler, or moving one that is there, with the pointer
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useCallback, useEffect, useRef, useState, type RefObject } from 'react';
import type { PlainRect } from './geometry';
import { overRuler, placeGuide, type GuideAxis } from './guides';
import { SNAP_THRESHOLD_PX, type SnapTargets } from './snap';

export interface GuideDragState {
	axis: GuideAxis;
	/** Where the guide would land, in points. */
	pos: number;
	/** The guide being moved, or `null` for one pulled out of a ruler. */
	index: number | null;
	/** True while the pointer is back over the ruler: releasing takes the guide away. */
	remove: boolean;
	/** Where the pointer is, in viewport pixels, for the tag that follows it. */
	pointer: { x: number; y: number };
}

export interface GuideDragOptions {
	/** The element the page scrolls in; the rulers sit above and left of it. */
	stage: RefObject<HTMLElement | null>;
	/** The spread's box with its bleed, in points. */
	bounds: PlainRect;
	/** Screen pixels per point. */
	scale: number;
	/** Where a guide snaps to, leaving out the guide being moved. */
	targets(moving: number | null): SnapTargets;
	/** The step a guide falls on when nothing nearer pulls it, in points. */
	fineStep: number;
	/** Whether to snap at all. Alt turns it off for one drag. */
	snap: boolean;
	onAdd(axis: GuideAxis, pos: number): void;
	onMove(index: number, pos: number): void;
	onRemove(index: number): void;
}

export function useGuideDrag(options: GuideDragOptions) {
	const [state, setState] = useState<GuideDragState | null>(null);
	const latest = useRef(options);
	latest.current = options;
	const active = useRef<{ stop(): void } | null>(null);

	// A drag is let go if the stage goes away in the middle of one.
	useEffect(() => () => active.current?.stop(), []);

	const begin = useCallback((axis: GuideAxis, index: number | null) => {
		active.current?.stop();

		const read = (event: PointerEvent): GuideDragState | null => {
			const { stage, bounds, scale, targets, fineStep, snap } = latest.current;
			const page = stage.current?.querySelector('svg[role="group"]');
			const room = stage.current?.getBoundingClientRect();
			if (!page || !room) return null;
			const box = page.getBoundingClientRect();
			const along =
				axis === 'x'
					? bounds.x + (event.clientX - box.left) / scale
					: bounds.y + (event.clientY - box.top) / scale;
			const pointer = { x: event.clientX, y: event.clientY };
			const placed = placeGuide(
				along,
				axis,
				targets(index),
				fineStep,
				SNAP_THRESHOLD_PX / scale,
				!snap || event.altKey,
			);
			return {
				axis,
				pos: placed.pos,
				index,
				remove: overRuler(axis, pointer, room),
				pointer,
			};
		};

		let last: GuideDragState | null = null;
		const onMove = (event: PointerEvent) => {
			last = read(event) ?? last;
			if (last) setState(last);
		};
		const finish = (commit: boolean) => {
			stop();
			const end = last;
			if (!commit || !end) return;
			const { onAdd, onMove: move, onRemove } = latest.current;
			if (end.index === null) {
				if (!end.remove) onAdd(end.axis, end.pos);
			} else if (end.remove) onRemove(end.index);
			else move(end.index, end.pos);
		};
		const onUp = () => finish(true);
		const onCancel = () => finish(false);
		const onKey = (event: KeyboardEvent) => {
			if (event.key === 'Escape') {
				event.preventDefault();
				finish(false);
			}
		};
		function stop() {
			window.removeEventListener('pointermove', onMove);
			window.removeEventListener('pointerup', onUp);
			window.removeEventListener('pointercancel', onCancel);
			window.removeEventListener('keydown', onKey);
			active.current = null;
			setState(null);
		}
		window.addEventListener('pointermove', onMove);
		window.addEventListener('pointerup', onUp);
		window.addEventListener('pointercancel', onCancel);
		window.addEventListener('keydown', onKey);
		active.current = { stop };
	}, []);

	return {
		state,
		/** A press on a ruler: the left one pulls out a vertical guide, the top one a horizontal. */
		fromRuler: (axis: 'x' | 'y') => (event: { button: number }) => {
			if (event.button !== 0) return;
			// The left ruler is the `y`-axis ruler, and makes guides at an x position.
			begin(axis === 'y' ? 'x' : 'y', null);
		},
		/** A press on a guide that is not locked. */
		fromGuide: (axis: GuideAxis, index: number) => (event: { button: number }) => {
			if (event.button !== 0) return;
			begin(axis, index);
		},
	};
}
