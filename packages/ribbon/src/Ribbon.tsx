// The ribbon: tabs, groups, controls, menus, tooltips and the keyboard model
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { ContextMenu } from '@liminal-hq/folio-chrome/ContextMenu';
import type { MenuItem, SelectableMenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import {
	Fragment,
	useEffect,
	useMemo,
	useRef,
	useState,
	type FocusEvent,
	type KeyboardEvent,
} from 'react';
import { defaultRibbonLabels, type RibbonLabels } from './labels';
import styles from './Ribbon.module.css';
import { RibbonControl } from './RibbonControls';
import type {
	RibbonCommand,
	RibbonDensity,
	RibbonGroup,
	RibbonItem,
	RibbonMode,
	RibbonModel,
	RibbonValue,
	RibbonVariant,
} from './types';
import { useRibbonTooltip } from './useRibbonTooltip';
import { visibleTabs } from './visibleTabs';

export interface RibbonProps {
	model: RibbonModel;
	mode: RibbonMode;
	onCommand: (command: RibbonCommand) => void;
	/** Ids of the contextual tabs to show. */
	contextual?: readonly string[];
	/** The active tab, when the app owns it. */
	tab?: string;
	variant?: RibbonVariant;
	density?: RibbonDensity;
	showLabels?: boolean;
	labels?: Partial<RibbonLabels>;
}

interface OpenMenu {
	/** The item that opened the menu, or `null` for the File button. */
	item: RibbonItem | null;
	items: MenuItem[];
	x: number;
	y: number;
}

/** Splits items into columns of `perColumn` small buttons; large buttons and other controls stand alone. */
function columns(items: RibbonItem[], perColumn: number): RibbonItem[][] {
	const out: RibbonItem[][] = [];
	let run: RibbonItem[] = [];
	const flush = () => {
		if (run.length > 0) out.push(run);
		run = [];
	};
	for (const item of items) {
		const small = (item.kind ?? 'button') !== 'select' && item.size !== 'large';
		if (!small) {
			flush();
			out.push([item]);
			continue;
		}
		run.push(item);
		if (run.length === perColumn) flush();
	}
	flush();
	return out;
}

function focusables(root: HTMLElement | null): HTMLElement[] {
	if (!root) return [];
	return Array.from(root.querySelectorAll<HTMLElement>('button, select')).filter(
		(el) => !(el as HTMLButtonElement).disabled,
	);
}

export function Ribbon(props: RibbonProps) {
	const {
		model,
		mode,
		onCommand,
		contextual = [],
		variant = 'full',
		density = 'comfortable',
		showLabels = true,
	} = props;
	const labels = { ...defaultRibbonLabels, ...props.labels };
	const tooltip = useRibbonTooltip();
	const [tabState, setTabState] = useState<string | null>(null);
	const [toggles, setToggles] = useState<Record<string, boolean>>({});
	const [values, setValues] = useState<Record<string, RibbonValue>>({});
	const [menu, setMenu] = useState<OpenMenu | null>(null);
	const [open, setOpen] = useState(false);
	const [pinned, setPinned] = useState(false);
	const tabsRef = useRef<HTMLDivElement>(null);
	const panelRef = useRef<HTMLDivElement>(null);

	const tabs = useMemo(() => visibleTabs(model, mode, contextual), [model, mode, contextual]);
	const wanted = props.tab ?? tabState;
	const active = tabs.find((t) => t.id === wanted) ?? tabs[0];
	// A remembered tab that has gone (a contextual tab whose subject was deselected) is forgotten,
	// so it is not restored the next time it reappears.
	const stale =
		props.tab === undefined && tabState !== null && !tabs.some((t) => t.id === tabState);
	useEffect(() => {
		if (stale) setTabState(null);
	}, [stale]);
	const collapsed = variant === 'collapsed';
	const simplified = variant === 'simplified';
	const panelShown = !collapsed || open || pinned;
	const labelled = showLabels && !simplified && density === 'comfortable';

	const emit = (command: RibbonCommand) => onCommand(command);

	/** Moving across tabs with the arrows only selects; activating a tab also drops the panel down or closes it. */
	const pickTab = (id: string, activate: boolean) => {
		setTabState(id);
		emit({ id: 'tab', value: id });
		if (collapsed && activate) setOpen(id !== active?.id || !open);
	};

	const onTabKeys = (event: KeyboardEvent) => {
		const buttons = Array.from(
			tabsRef.current?.querySelectorAll<HTMLElement>('[role="tab"]') ?? [],
		);
		const at = buttons.indexOf(document.activeElement as HTMLElement);
		if (at < 0) return;
		let next = -1;
		if (event.key === 'ArrowRight') next = (at + 1) % buttons.length;
		else if (event.key === 'ArrowLeft') next = (at - 1 + buttons.length) % buttons.length;
		else if (event.key === 'Home') next = 0;
		else if (event.key === 'End') next = buttons.length - 1;
		else if (event.key === 'ArrowDown') {
			focusables(panelRef.current)[0]?.focus();
			event.preventDefault();
			return;
		}
		if (next < 0) return;
		event.preventDefault();
		buttons[next]?.focus();
		const id = tabs[next]?.id;
		if (id) pickTab(id, false);
	};

	// Roving tabindex: one control in the panel is a Tab stop, the arrows move it.
	const roving = useRef<HTMLElement | null>(null);
	useEffect(() => {
		const all = focusables(panelRef.current);
		const stop = roving.current && all.includes(roving.current) ? roving.current : all[0];
		for (const el of all) el.tabIndex = el === stop ? 0 : -1;
	});
	const onPanelFocus = (event: FocusEvent<HTMLElement>) => {
		const target = event.target as HTMLElement;
		if (!focusables(panelRef.current).includes(target)) return;
		roving.current = target;
		for (const el of focusables(panelRef.current)) el.tabIndex = el === target ? 0 : -1;
	};

	const onPanelKeys = (event: KeyboardEvent) => {
		if (event.key !== 'ArrowRight' && event.key !== 'ArrowLeft') return;
		const all = focusables(panelRef.current);
		const at = all.indexOf(document.activeElement as HTMLElement);
		if (at < 0 || (document.activeElement as HTMLElement).tagName === 'SELECT') return;
		event.preventDefault();
		const step = event.key === 'ArrowRight' ? 1 : -1;
		all[(at + step + all.length) % all.length]?.focus();
	};

	/** Esc closes a dropped-down collapsed ribbon and returns to the tab, from anywhere inside it. */
	const onRootKeys = (event: KeyboardEvent) => {
		if (event.key !== 'Escape' || event.defaultPrevented || menu) return;
		const inPanel = panelRef.current?.contains(document.activeElement);
		if (collapsed && open) setOpen(false);
		else if (!inPanel) return;
		event.preventDefault();
		tabsRef.current?.querySelector<HTMLElement>('[role="tab"][aria-selected="true"]')?.focus();
	};

	/** Ribbon-kept state is keyed by tab, group and id, so two controls sharing an id do not share state. */
	const keyOf = (tabId: string, groupId: string, item: RibbonItem) =>
		`${tabId}/${groupId}/${item.id}`;
	const toggleOn = (key: string, item: RibbonItem) => item.active ?? toggles[key] ?? false;

	const onPress = (key: string, item: RibbonItem, anchor: HTMLElement) => {
		const kind = item.kind ?? 'button';
		if (kind === 'menu') {
			const rect = anchor.getBoundingClientRect();
			setMenu({
				item,
				x: rect.left,
				y: rect.bottom,
				items: (item.options ?? []).map(([value, label]) => ({
					type: 'action',
					id: value,
					label,
				})),
			});
			return;
		}
		if (kind === 'toggle') {
			const next = !toggleOn(key, item);
			setToggles((prev) => {
				const out = { ...prev, [key]: next };
				if (item.radio && next) {
					for (const tab of model.tabs)
						for (const group of tab.groups) {
							const peers = [...(group.items ?? []), ...(group.rows ?? []).flat(2)];
							for (const peer of peers) {
								const peerKey = keyOf(tab.id, group.id, peer);
								if (peer.radio === item.radio && peerKey !== key) out[peerKey] = false;
							}
						}
				}
				return out;
			});
			emit({ id: item.id, value: next, item });
			return;
		}
		emit({ id: item.id, item });
	};

	const onValue = (key: string, item: RibbonItem, value: RibbonValue) => {
		setValues((prev) => ({ ...prev, [key]: value }));
		emit({ id: item.id, value, item });
	};

	const openFile = (anchor: HTMLElement) => {
		const items = model.file?.items;
		if (!items || items.length === 0) {
			emit({ id: 'file' });
			return;
		}
		const rect = anchor.getBoundingClientRect();
		setMenu({
			item: null,
			x: rect.left,
			y: rect.bottom,
			items: items.map(([value, label]) => ({ type: 'action', id: value, label })),
		});
	};

	const onMenuSelect = (picked: SelectableMenuItem) => {
		if (menu?.item) emit({ id: menu.item.id, value: picked.id, item: menu.item });
		else emit({ id: 'file', value: picked.id });
		setMenu(null);
	};

	const control = (group: RibbonGroup) => {
		const Rendered = (item: RibbonItem) => {
			const key = keyOf(active?.id ?? '', group.id, item);
			return (
				<RibbonControl
					key={item.id}
					item={item}
					showLabel={labelled}
					tooltip={tooltip}
					pressed={toggleOn(key, item)}
					value={item.value ?? values[key]}
					labels={labels}
					onPress={(i, anchor) => onPress(key, i, anchor)}
					onValue={(i, value) => onValue(key, i, value)}
				/>
			);
		};
		return Rendered;
	};

	const perColumn = density === 'compact' ? 2 : 3;
	const renderGroup = (group: RibbonGroup) => (
		<div key={group.id} role="group" aria-label={group.label} className={styles.group}>
			<div className={styles.groupBody}>
				{group.rows ? (
					<div className={styles.rows}>
						{group.rows.map((row, r) => (
							<div key={r} className={styles.row}>
								{row.map((cluster, c) => (
									<div key={c} className={styles.cluster}>
										{cluster.map(control(group))}
									</div>
								))}
							</div>
						))}
					</div>
				) : (
					columns(group.items ?? [], perColumn).map((col, i) => (
						<div key={i} className={styles.column}>
							{col.map(control(group))}
						</div>
					))
				)}
			</div>
			{density === 'comfortable' && !simplified && (
				<span className={styles.groupLabel}>{group.label}</span>
			)}
		</div>
	);

	return (
		<div
			role="group"
			aria-label={labels.ribbon}
			className={styles.ribbon}
			data-variant={variant}
			data-density={density}
			onKeyDown={onRootKeys}
		>
			<div className={styles.tabRow}>
				{model.file && (
					<button
						type="button"
						className={styles.file}
						aria-haspopup={model.file.items?.length ? 'menu' : undefined}
						onClick={(e) => openFile(e.currentTarget)}
					>
						{model.file.label}
					</button>
				)}
				<div
					ref={tabsRef}
					role="tablist"
					aria-label={labels.tabs}
					className={styles.tabs}
					onKeyDown={onTabKeys}
				>
					{tabs.map((tab) => (
						<button
							key={tab.id}
							type="button"
							role="tab"
							id={`folio-ribbon-tab-${tab.id}`}
							aria-selected={tab.id === active?.id}
							aria-controls="folio-ribbon-panel"
							tabIndex={tab.id === active?.id ? 0 : -1}
							className={styles.tab}
							data-contextual={tab.contextual ? '' : undefined}
							onClick={() => pickTab(tab.id, true)}
						>
							{tab.label}
						</button>
					))}
				</div>
				{simplified && (
					<div
						className={styles.inlinePanel}
						role="toolbar"
						aria-label={labels.toolbar}
						onKeyDown={onPanelKeys}
						onFocus={onPanelFocus}
						ref={panelRef}
					>
						{active?.groups.map((group) => (
							<Fragment key={group.id}>{renderGroup(group)}</Fragment>
						))}
					</div>
				)}
				<div className={styles.end}>
					{collapsed && (
						<button
							type="button"
							className={styles.pin}
							aria-pressed={pinned}
							onClick={() => setPinned((p) => !p)}
						>
							{pinned ? labels.unpin : labels.pin}
						</button>
					)}
					<div role="radiogroup" aria-label={labels.mode} className={styles.mode}>
						{(['easy', 'expert'] as const).map((m) => (
							<button
								key={m}
								type="button"
								role="radio"
								aria-checked={mode === m}
								className={styles.modeOption}
								onClick={() => mode !== m && emit({ id: 'mode', value: m })}
							>
								{m === 'easy' ? labels.modeEasy : labels.modeExpert}
							</button>
						))}
					</div>
				</div>
			</div>
			{!simplified && panelShown && active && (
				<div
					ref={panelRef}
					role="tabpanel"
					id="folio-ribbon-panel"
					aria-labelledby={`folio-ribbon-tab-${active.id}`}
					className={styles.panel}
					data-overlay={collapsed && !pinned ? '' : undefined}
					onKeyDown={onPanelKeys}
					onFocus={onPanelFocus}
				>
					<div role="toolbar" aria-label={labels.toolbar} className={styles.toolbar}>
						{active.groups.map(renderGroup)}
					</div>
				</div>
			)}
			{tooltip.node}
			{menu && (
				<ContextMenu
					items={menu.items}
					position={{ x: menu.x, y: menu.y }}
					ariaLabel={menu.item?.label ?? model.file?.label}
					onSelect={onMenuSelect}
					onClose={() => setMenu(null)}
				/>
			)}
		</div>
	);
}
