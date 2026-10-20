// The grouped sidebar: a card on the right with a tab for each group and the group's panel under it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useRef, type KeyboardEvent, type ReactNode } from 'react';
import { t } from '../i18n/messages';
import styles from './Sidebar.module.css';

export type SidebarGroup = 'properties' | 'pages' | 'proofing' | 'fonts' | 'swatches';

const GROUPS: readonly SidebarGroup[] = ['properties', 'pages', 'proofing', 'fonts'];
/** The groups of Expert mode: the same, and the book's swatches. */
export const EXPERT_GROUPS: readonly SidebarGroup[] = [...GROUPS, 'swatches'];

export interface SidebarProps {
	group: SidebarGroup;
	onGroup(group: SidebarGroup): void;
	/** The panel of each group; only the open group's is drawn. */
	panels: Partial<Record<SidebarGroup, ReactNode>>;
	/** The groups to offer, in order; the four of Easy mode by default. */
	groups?: readonly SidebarGroup[];
}

/**
 * The groups of spec 12 that the book has panels for. The tabs are a `tablist` (arrow keys, Home and
 * End move between them, and the open one is the one tab stop); each panel is the `tabpanel` they
 * label.
 */
export function Sidebar({ group, onGroup, panels, groups = GROUPS }: SidebarProps) {
	const tabs = useRef<Array<HTMLButtonElement | null>>([]);

	const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
		const at = groups.indexOf(group);
		let next = at;
		if (event.key === 'ArrowRight') next = (at + 1) % groups.length;
		else if (event.key === 'ArrowLeft') next = (at - 1 + GROUPS.length) % groups.length;
		else if (event.key === 'Home') next = 0;
		else if (event.key === 'End') next = groups.length - 1;
		else return;
		event.preventDefault();
		onGroup(groups[next]!);
		tabs.current[next]?.focus();
	};

	return (
		<aside className={styles.card} aria-label={t('region.sidebar')}>
			<div
				role="tablist"
				aria-label={t('region.sidebar')}
				className={styles.tabs}
				onKeyDown={onKeyDown}
			>
				{groups.map((id, index) => (
					<button
						key={id}
						ref={(element) => {
							tabs.current[index] = element;
						}}
						type="button"
						role="tab"
						id={`sidebar-tab-${id}`}
						aria-selected={id === group}
						aria-controls={`sidebar-panel-${id}`}
						tabIndex={id === group ? 0 : -1}
						className={styles.tab}
						onClick={() => onGroup(id)}
					>
						{t(`sidebar.${id}`)}
					</button>
				))}
			</div>
			<div
				role="tabpanel"
				id={`sidebar-panel-${group}`}
				aria-labelledby={`sidebar-tab-${group}`}
				className={styles.panel}
			>
				{panels[group]}
			</div>
		</aside>
	);
}
