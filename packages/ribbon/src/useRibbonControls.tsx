// The state and menus behind the ribbon's controls, shared by the ribbon and the classic toolbar
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { ContextMenu } from '@liminal-hq/folio-chrome/ContextMenu';
import type { MenuItem, SelectableMenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import { useState, type ReactNode } from 'react';
import type { RibbonLabels } from './labels';
import { RibbonControl } from './RibbonControls';
import type { RibbonCommand, RibbonItem, RibbonValue } from './types';
import { useRibbonTooltip } from './useRibbonTooltip';

interface OpenMenu {
	/** The item that opened the menu, or `null` for the File button. */
	item: RibbonItem | null;
	label: string | undefined;
	items: MenuItem[];
	x: number;
	y: number;
}

/** An item that can share a `radio` key, with the scope (its group) that keeps its state apart from same-id items elsewhere. */
export interface RibbonPeer {
	scope: string;
	item: RibbonItem;
}

const keyOf = (scope: string, item: RibbonItem) => `${scope}/${item.id}`;

export interface RibbonControls {
	/** Draws one item as its control; state is kept per `scope` and id, so same-id controls in different scopes stay independent. */
	control: (item: RibbonItem, showLabel: boolean, scope?: string) => ReactNode;
	/** Opens the File menu under `anchor`. */
	openFile: (anchor: HTMLElement, label: string, items: [string, string][]) => void;
	/** The tooltip and the open menu, to render once inside the ribbon. */
	overlays: ReactNode;
}

/**
 * Toggles and values are kept here unless an item sets `active` or `value`, which then owns them.
 * `peers` lists every item that can share a `radio` key with another.
 */
export function useRibbonControls(
	peers: readonly RibbonPeer[],
	labels: RibbonLabels,
	emit: (command: RibbonCommand) => void,
): RibbonControls {
	const tooltip = useRibbonTooltip();
	const [toggles, setToggles] = useState<Record<string, boolean>>({});
	const [values, setValues] = useState<Record<string, RibbonValue>>({});
	const [menu, setMenu] = useState<OpenMenu | null>(null);

	const toggleOn = (scope: string, item: RibbonItem) =>
		item.active ?? toggles[keyOf(scope, item)] ?? false;
	const valueOf = (scope: string, item: RibbonItem) => item.value ?? values[keyOf(scope, item)];

	const onPress = (scope: string, item: RibbonItem, anchor: HTMLElement) => {
		const kind = item.kind ?? 'button';
		if (kind === 'menu') {
			const rect = anchor.getBoundingClientRect();
			setMenu({
				item,
				label: item.label,
				x: rect.left,
				y: rect.bottom,
				items: (item.options ?? []).map(([value, label]) => ({ type: 'action', id: value, label })),
			});
			return;
		}
		if (kind === 'toggle') {
			const next = !toggleOn(scope, item);
			setToggles((prev) => {
				const key = keyOf(scope, item);
				const out = { ...prev, [key]: next };
				if (item.radio && next) {
					for (const peer of peers) {
						const peerKey = keyOf(peer.scope, peer.item);
						if (peer.item.radio === item.radio && peerKey !== key) out[peerKey] = false;
					}
				}
				return out;
			});
			emit({ id: item.id, value: next, item });
			return;
		}
		emit({ id: item.id, item });
	};

	const onValue = (scope: string, item: RibbonItem, value: RibbonValue) => {
		setValues((prev) => ({ ...prev, [keyOf(scope, item)]: value }));
		emit({ id: item.id, value, item });
	};

	const onMenuSelect = (picked: SelectableMenuItem) => {
		if (menu?.item) emit({ id: menu.item.id, value: picked.id, item: menu.item });
		else emit({ id: 'file', value: picked.id });
		setMenu(null);
	};

	return {
		control: (item, showLabel, scope = '') => (
			<RibbonControl
				key={item.id}
				item={item}
				showLabel={showLabel}
				tooltip={tooltip}
				pressed={toggleOn(scope, item)}
				value={valueOf(scope, item)}
				labels={labels}
				onPress={(i, anchor) => onPress(scope, i, anchor)}
				onValue={(i, value) => onValue(scope, i, value)}
			/>
		),
		openFile: (anchor, label, items) => {
			const rect = anchor.getBoundingClientRect();
			setMenu({
				item: null,
				label,
				x: rect.left,
				y: rect.bottom,
				items: items.map(([value, text]) => ({ type: 'action', id: value, label: text })),
			});
		},
		overlays: (
			<>
				{tooltip.node}
				{menu && (
					<ContextMenu
						items={menu.items}
						position={{ x: menu.x, y: menu.y }}
						ariaLabel={menu.label}
						onSelect={onMenuSelect}
						onClose={() => setMenu(null)}
					/>
				)}
			</>
		),
	};
}
