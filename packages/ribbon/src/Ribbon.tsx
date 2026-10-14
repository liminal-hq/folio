// The ribbon: tabs, groups, controls, menus, tooltips and the keyboard model
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Fragment, useMemo, useRef, useState, type KeyboardEvent } from 'react';
import { defaultRibbonLabels, type RibbonLabels } from './labels';
import styles from './Ribbon.module.css';
import { RibbonModeSwitch } from './RibbonModeSwitch';
import type {
	RibbonCommand,
	RibbonDensity,
	RibbonGroup,
	RibbonItem,
	RibbonMode,
	RibbonModel,
	RibbonVariant,
} from './types';
import { useRibbonControls } from './useRibbonControls';
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
	const [tabState, setTabState] = useState<string | null>(null);
	const [open, setOpen] = useState(false);
	const [pinned, setPinned] = useState(false);
	const tabsRef = useRef<HTMLDivElement>(null);
	const panelRef = useRef<HTMLDivElement>(null);

	const tabs = useMemo(() => visibleTabs(model, mode, contextual), [model, mode, contextual]);
	const wanted = props.tab ?? tabState;
	const active = tabs.find((t) => t.id === wanted) ?? tabs[0];
	const collapsed = variant === 'collapsed';
	const simplified = variant === 'simplified';
	const panelShown = !collapsed || open || pinned;
	const labelled = showLabels && !simplified && density === 'comfortable';

	const peers = useMemo(
		() =>
			model.tabs.flatMap((tab) =>
				tab.groups.flatMap((group) => [...(group.items ?? []), ...(group.rows ?? []).flat(2)]),
			),
		[model],
	);
	const emit = (command: RibbonCommand) => onCommand(command);
	const controls = useRibbonControls(peers, labels, emit);

	const pickTab = (id: string) => {
		setTabState(id);
		emit({ id: 'tab', value: id });
		if (collapsed) setOpen(id !== active?.id || !open);
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
		if (id) pickTab(id);
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
		if (event.key !== 'Escape' || event.defaultPrevented) return;
		const inPanel = panelRef.current?.contains(document.activeElement);
		if (collapsed && open) setOpen(false);
		else if (!inPanel) return;
		event.preventDefault();
		tabsRef.current?.querySelector<HTMLElement>('[role="tab"][aria-selected="true"]')?.focus();
	};

	const openFile = (anchor: HTMLElement) => {
		const items = model.file?.items;
		if (!model.file || !items || items.length === 0) emit({ id: 'file' });
		else controls.openFile(anchor, model.file.label, items);
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
										{cluster.map((item) => controls.control(item, false))}
									</div>
								))}
							</div>
						))}
					</div>
				) : (
					columns(group.items ?? [], perColumn).map((col, i) => (
						<div key={i} className={styles.column}>
							{col.map((item) => controls.control(item, labelled))}
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
			role="toolbar"
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
							onClick={() => pickTab(tab.id)}
						>
							{tab.label}
						</button>
					))}
				</div>
				{simplified && (
					<div
						className={styles.inlinePanel}
						role="group"
						aria-label={active?.label}
						onKeyDown={onPanelKeys}
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
					<RibbonModeSwitch
						mode={mode}
						labels={labels}
						onChange={(next) => emit({ id: 'mode', value: next })}
					/>
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
				>
					{active.groups.map(renderGroup)}
				</div>
			)}
			{controls.overlays}
		</div>
	);
}
