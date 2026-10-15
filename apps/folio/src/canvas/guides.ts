// Ruler guides: where a dragged one lands, when it is dropped back on its ruler, and the menu's intents
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { isMasterSpread, surfaceOf } from './masterView';
import { snapAxis, type SnapTargets } from './snap';

export type GuideAxis = 'x' | 'y';

export interface Placed {
	pos: number;
	/** The edge, centre or guide it snapped to, if it did. */
	to: number | null;
}

/**
 * Where a guide dragged to `raw` lands, in points: on a margin, frame edge or other guide that is
 * within `threshold`, else on the unit's `fineStep`. `free` (Alt) leaves it where the pointer is.
 */
export function placeGuide(
	raw: number,
	axis: GuideAxis,
	targets: SnapTargets,
	fineStep: number,
	threshold: number,
	free: boolean,
): Placed {
	if (free) return { pos: raw, to: null };
	const near = snapAxis([raw], targets[axis], threshold);
	if (near) return { pos: near.target, to: near.target };
	return { pos: Math.round(raw / fineStep) * fineStep, to: null };
}

/**
 * Whether a pointer is back over the ruler a guide of `axis` came from. A vertical guide (`x`) is
 * pulled from the left ruler, so it is over it left of the stage; a horizontal one is over the top
 * ruler above the stage.
 */
export function overRuler(
	axis: GuideAxis,
	pointer: { x: number; y: number },
	stage: { left: number; top: number },
): boolean {
	return axis === 'x' ? pointer.x < stage.left : pointer.y < stage.top;
}

export type GuideCommand =
	| 'lock'
	| 'unlock'
	| 'delete'
	| 'clearSpread'
	| 'clearAll'
	/** Moves a spread's guide onto the masters, so it shows on every page. */
	| 'promote'
	/** Moves a master's guide back onto one spread. */
	| 'demote';

/** The intents a choice from a guide's menu makes. */
export function guideIntents(
	command: GuideCommand,
	spread: Spread,
	index: number,
	/** The spread a master's guide goes back onto, while a master is edited. */
	demoteTo: string | null = null,
): Intent[] {
	const on = { on: surfaceOf(spread) };
	// A master's guides are cleared one at a time for now.
	const master = isMasterSpread(spread);
	switch (command) {
		case 'lock':
			return [{ type: 'LockGuide', ...on, index, locked: true }];
		case 'unlock':
			return [{ type: 'LockGuide', ...on, index, locked: false }];
		case 'delete':
			return [{ type: 'RemoveGuide', ...on, index }];
		case 'clearSpread':
			return master ? [] : [{ type: 'ClearGuides', spread: spread.id }];
		case 'clearAll':
			return master ? [] : [{ type: 'ClearGuides' }];
		case 'promote':
			return master ? [] : [{ type: 'PromoteGuide', spread: spread.id, index }];
		case 'demote': {
			const surface = surfaceOf(spread);
			return 'master' in surface && demoteTo
				? [{ type: 'DemoteGuide', master: surface.master, index, spread: demoteTo }]
				: [];
		}
	}
}
