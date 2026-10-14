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

export interface RibbonControls {
	/** Draws one item as its control. */
	control: (item: RibbonItem, showLabel: boolean) => ReactNode;
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
	peers: readonly RibbonItem[],
	labels: RibbonLabels,
	emit: (command: RibbonCommand) => void,
): RibbonControls {
	const tooltip = useRibbonTooltip();
	const [toggles, setToggles] = useState<Record<string, boolean>>({});
	const [values, setValues] = useState<Record<string, RibbonValue>>({});
	const [menu, setMenu] = useState<OpenMenu | null>(null);

	const toggleOn = (item: RibbonItem) => item.active ?? toggles[item.id] ?? false;
	const valueOf = (item: RibbonItem) => item.value ?? values[item.id];

	const onPress = (item: RibbonItem, anchor: HTMLElement) => {
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
			const next = !toggleOn(item);
			setToggles((prev) => {
				const out = { ...prev, [item.id]: next };
				if (item.radio && next) {
					for (const peer of peers) {
						if (peer.radio === item.radio && peer.id !== item.id) out[peer.id] = false;
					}
				}
				return out;
			});
			emit({ id: item.id, value: next, item });
			return;
		}
		emit({ id: item.id, item });
	};

	const onValue = (item: RibbonItem, value: RibbonValue) => {
		setValues((prev) => ({ ...prev, [item.id]: value }));
		emit({ id: item.id, value, item });
	};

	const onMenuSelect = (picked: SelectableMenuItem) => {
		if (menu?.item) emit({ id: menu.item.id, value: picked.id, item: menu.item });
		else emit({ id: 'file', value: picked.id });
		setMenu(null);
	};

	return {
		control: (item, showLabel) => (
			<RibbonControl
				key={item.id}
				item={item}
				showLabel={showLabel}
				tooltip={tooltip}
				pressed={toggleOn(item)}
				value={valueOf(item)}
				labels={labels}
				onPress={onPress}
				onValue={onValue}
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
