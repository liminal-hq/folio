// Which tabs, groups and items show for a mode and a set of contextual tabs
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { RibbonGroup, RibbonItem, RibbonMode, RibbonModel, RibbonTab } from './types';

function visibleItems(items: RibbonItem[], mode: RibbonMode): RibbonItem[] {
	return items.filter((item) => mode === 'expert' || !item.expert);
}

function visibleGroup(group: RibbonGroup, mode: RibbonMode): RibbonGroup | null {
	if (group.expert && mode === 'easy') return null;
	const items = group.items ? visibleItems(group.items, mode) : undefined;
	const rows = group.rows
		?.map((row) => row.map((cluster) => visibleItems(cluster, mode)).filter((c) => c.length > 0))
		.filter((row) => row.length > 0);
	if ((items?.length ?? 0) === 0 && (rows?.length ?? 0) === 0) return null;
	return { ...group, items, rows };
}

/**
 * The tabs to draw: contextual tabs only when listed in `contextual`, expert groups and items only in
 * Expert mode, and no group left empty. A tab with nothing to show is dropped.
 */
export function visibleTabs(
	model: RibbonModel,
	mode: RibbonMode,
	contextual: readonly string[] = [],
): RibbonTab[] {
	return model.tabs
		.filter((tab) => !tab.contextual || contextual.includes(tab.id))
		.map((tab) => ({
			...tab,
			groups: tab.groups
				.map((group) => visibleGroup(group, mode))
				.filter((g): g is RibbonGroup => g !== null),
		}))
		.filter((tab) => tab.groups.length > 0);
}
