// The page marks: which there are, their four curated colours each, and how they sit in settings
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MarksSettings } from '@liminal-hq/folio-protocol/generated/MarksSettings';
import type { SettingsPatch } from '@liminal-hq/folio-plugin-settings';

export type MarkKey = 'bleed' | 'margins' | 'columns' | 'baseline' | 'frameEdges';

export const MARK_KEYS: readonly MarkKey[] = [
	'bleed',
	'margins',
	'columns',
	'baseline',
	'frameEdges',
];

/** Which marks are drawn. */
export type Marks = Record<MarkKey, boolean>;

/** The colour each mark is drawn in. */
export type MarkColours = Record<MarkKey, string>;

type Palette = readonly [string, string, string, string];

/** The four colours each mark can take; the first is the default. Marks never print. */
export const MARK_PALETTES: Readonly<Record<MarkKey, Palette>> = {
	bleed: ['#d9534f', '#e0368f', '#7b4fd6', '#3a3a3a'],
	margins: ['#5b7fd6', '#2f9e7a', '#7b4fd6', '#3a3a3a'],
	columns: ['#ff8a1f', '#e0368f', '#2f9e7a', '#5b7fd6'],
	baseline: ['#d95fa0', '#5b7fd6', '#2f9e7a', '#ff8a1f'],
	frameEdges: ['#2f9e7a', '#5b7fd6', '#e0368f', '#ff8a1f'],
};

export const DEFAULT_MARKS: Marks = {
	bleed: true,
	margins: true,
	columns: false,
	baseline: false,
	frameEdges: false,
};

export const DEFAULT_MARK_COLOURS: MarkColours = {
	bleed: MARK_PALETTES.bleed[0],
	margins: MARK_PALETTES.margins[0],
	columns: MARK_PALETTES.columns[0],
	baseline: MARK_PALETTES.baseline[0],
	frameEdges: MARK_PALETTES.frameEdges[0],
};

export function marksOf(settings: MarksSettings): Marks {
	return Object.fromEntries(MARK_KEYS.map((key) => [key, settings[key].on])) as Marks;
}

export function coloursOf(settings: MarksSettings): MarkColours {
	return Object.fromEntries(
		MARK_KEYS.map((key) => [
			key,
			MARK_PALETTES[key][settings[key].colour] ?? MARK_PALETTES[key][0],
		]),
	) as MarkColours;
}

/** The settings change that makes the marks `next`, naming only the marks that differ from `now`. */
export function marksPatch(now: Marks, next: Partial<Marks>): SettingsPatch {
	const marks: Record<string, { on: boolean }> = {};
	for (const key of MARK_KEYS) {
		const wanted = next[key];
		if (wanted !== undefined && wanted !== now[key]) marks[key] = { on: wanted };
	}
	return { marks } as SettingsPatch;
}
