// How the ribbon's groups give way when the panel is narrower than they are
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/**
 * How a group is drawn: whole, with its buttons as icons only, or folded into one button that
 * drops the group down.
 */
export type GroupLevel = 'full' | 'compact' | 'folded';

/** How the groups of the panel are drawn: each group's level, and how many at the end go under More. */
export interface RibbonFit {
	levels: Readonly<Record<string, GroupLevel>>;
	/** The groups at the end of the panel that are only under the More button. */
	more: number;
}

export const WHOLE: RibbonFit = { levels: {}, more: 0 };

/** One group as fitting sees it: whether drawing its buttons as icons only makes it narrower. */
export interface FitGroup {
	id: string;
	compactable: boolean;
}

/**
 * The next way to make the panel narrower, or `null` when there is none left. Groups give way from
 * the right, as an office ribbon's do: each in turn to icons only, then each in turn folded into
 * one button, and then the last groups go under a More button, one at a time.
 */
export function narrower(fit: RibbonFit, groups: readonly FitGroup[]): RibbonFit | null {
	const shown = groups.slice(0, groups.length - fit.more);
	const level = (id: string) => fit.levels[id] ?? 'full';
	for (let i = shown.length - 1; i >= 0; i -= 1) {
		const group = shown[i]!;
		if (group.compactable && level(group.id) === 'full')
			return { ...fit, levels: { ...fit.levels, [group.id]: 'compact' } };
	}
	for (let i = shown.length - 1; i >= 0; i -= 1) {
		const group = shown[i]!;
		if (level(group.id) !== 'folded')
			return { ...fit, levels: { ...fit.levels, [group.id]: 'folded' } };
	}
	if (fit.more < groups.length) return { ...fit, more: fit.more + 1 };
	return null;
}
