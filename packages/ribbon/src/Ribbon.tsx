// The ribbon: tabs, groups, controls, menus, tooltips and the keyboard model
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import {
	Fragment,
	useEffect,
	useLayoutEffect,
	useMemo,
	useRef,
	useState,
	type CSSProperties,
	type FocusEvent,
	type KeyboardEvent,
	type ReactNode,
} from 'react';
import { defaultRibbonLabels, type RibbonLabels } from './labels';
import styles from './Ribbon.module.css';
import { Glyph } from './RibbonControls';
import { WHOLE, narrower, type GroupLevel, type RibbonFit } from './ribbonFit';
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
	/** Shown at the right end, before the Easy and Expert switch. */
	end?: ReactNode;
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

/** Three dots: the More button's glyph. */
const MORE_GLYPH = 'M3.5 8h.01M8 8h.01M12.5 8h.01';

/** Every item of a group, in order, whichever way it is laid out. */
const itemsOf = (group: RibbonGroup): RibbonItem[] => group.items ?? (group.rows ?? []).flat(2);

/** A group drawn with its buttons as icons only: small, with no words beside them. */
const compacted = (group: RibbonGroup): RibbonGroup =>
	group.items
		? { ...group, items: group.items.map((item) => ({ ...item, size: 'small' as const })) }
		: group;

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

	const peers = useMemo(
		() =>
			model.tabs.flatMap((tab) =>
				tab.groups.flatMap((group) =>
					[...(group.items ?? []), ...(group.rows ?? []).flat(2)].map((item) => ({
						scope: `${tab.id}/${group.id}`,
						item,
					})),
				),
			),
		[model],
	);
	const emit = (command: RibbonCommand) => onCommand(command);
	const controls = useRibbonControls(peers, labels, emit);

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
		const all = focusables(panelRef.current);
		if (!all.includes(target)) return;
		roving.current = target;
		for (const el of all) el.tabIndex = el === target ? 0 : -1;
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

	// The panel never scrolls: when the groups are wider than it, they give way from the right (to
	// icons only, then folded into one button each, then under More) until they fit. The fit is worked
	// out again from the whole groups whenever the panel's width or what it shows changes.
	const [width, setWidth] = useState(0);
	const groups = active?.groups ?? [];
	const fitKey = [
		active?.id,
		mode,
		density,
		variant,
		labelled,
		width,
		...groups.map((g) => g.id),
	].join('|');
	const [fitted, setFitted] = useState<{ key: string; fit: RibbonFit }>({ key: '', fit: WHOLE });
	const fit = fitted.key === fitKey ? fitted.fit : WHOLE;
	const fitGroups = useMemo(
		() =>
			groups.map((g) => ({
				id: g.id,
				compactable: !!g.items && (labelled || g.items.some((i) => i.size === 'large')),
			})),
		[groups, labelled],
	);
	useLayoutEffect(() => {
		const panel = panelRef.current;
		if (!panel || simplified || panel.scrollWidth <= panel.clientWidth) return;
		const next = narrower(fit, fitGroups);
		if (next) setFitted({ key: fitKey, fit: next });
	});
	useEffect(() => {
		const panel = panelRef.current;
		if (!panel || simplified || typeof ResizeObserver === 'undefined') return;
		const observer = new ResizeObserver(([entry]) => {
			if (entry) setWidth(Math.round(entry.contentRect.width));
		});
		observer.observe(panel);
		return () => observer.disconnect();
	}, [panelShown, simplified, active?.id]);

	/** The groups dropped down under a folded group's button or More, and where. */
	const [popup, setPopup] = useState<{
		key: string;
		ids: string[];
		label: string;
		x: number;
		y: number;
	} | null>(null);
	const popupRef = useRef<HTMLDivElement>(null);
	const openPopup = (key: string, ids: string[], label: string, anchor: HTMLElement) => {
		if (popup?.key === key) {
			setPopup(null);
			return;
		}
		const rect = anchor.getBoundingClientRect();
		setPopup({ key, ids, label, x: rect.left, y: rect.bottom });
	};
	// What the panel shows changed under it: the groups it dropped down may be whole again.
	useEffect(() => setPopup(null), [fitKey]);
	useEffect(() => {
		if (!popup) return;
		const away = (event: Event) => {
			const target = event.target;
			if (!(target instanceof Element)) return;
			if (popupRef.current?.contains(target) || target.closest('[data-folio-menu-root]')) return;
			if (target.closest(`[data-ribbon-popup-key="${popup.key}"]`)) return;
			setPopup(null);
		};
		// Escape closes the drop-down first, wherever the focus is (the words being edited keep it).
		const escape = (event: globalThis.KeyboardEvent) => {
			if (event.key !== 'Escape') return;
			event.preventDefault();
			event.stopPropagation();
			setPopup(null);
		};
		document.addEventListener('pointerdown', away, true);
		window.addEventListener('keydown', escape, true);
		return () => {
			document.removeEventListener('pointerdown', away, true);
			window.removeEventListener('keydown', escape, true);
		};
	}, [popup]);
	// A drop-down kept inside the window.
	useLayoutEffect(() => {
		const el = popupRef.current;
		if (!el || !popup) return;
		const room = window.innerWidth - el.offsetWidth - 8;
		if (popup.x > room) el.style.left = `${Math.max(8, room)}px`;
	}, [popup]);

	const perColumn = density === 'compact' ? 2 : 3;
	/** The button a group is folded into, or More: the group's name, which drops it down. */
	const dropButton = (key: string, label: string, d: string, ids: string[], disabled: boolean) => (
		<button
			type="button"
			className={`${styles.button} ${styles.large} ${styles.fold}`}
			aria-haspopup="true"
			aria-expanded={popup?.key === key}
			data-ribbon-popup-key={key}
			disabled={disabled}
			// The words being edited keep the focus, as with every other button of the ribbon.
			onMouseDown={(e) => e.preventDefault()}
			onClick={(e) => openPopup(key, ids, label, e.currentTarget)}
		>
			<Glyph d={d} px={22} />
			<span className={styles.label}>
				{label}
				<span className={styles.caret} aria-hidden="true">
					{' '}
					▾
				</span>
			</span>
		</button>
	);
	const renderGroup = (whole: RibbonGroup, level: GroupLevel = 'full') => {
		if (level === 'folded') {
			const items = itemsOf(whole);
			return (
				<div
					key={whole.id}
					role="group"
					aria-label={whole.label}
					className={styles.group}
					data-level="folded"
				>
					{dropButton(
						`group:${whole.id}`,
						whole.label,
						items[0]?.d ?? MORE_GLYPH,
						[whole.id],
						items.every((i) => i.disabled),
					)}
				</div>
			);
		}
		const group = level === 'compact' ? compacted(whole) : whole;
		const showLabels = labelled && level === 'full';
		const scope = `${active?.id ?? ''}/${group.id}`;
		return (
			<div
				key={group.id}
				role="group"
				aria-label={group.label}
				className={styles.group}
				data-level={level === 'full' ? undefined : level}
			>
				<div className={styles.groupBody}>
					{group.rows ? (
						<div className={styles.rows}>
							{group.rows.map((row, r) => (
								<div key={r} className={styles.row}>
									{row.map((cluster, c) => (
										<div key={c} className={styles.cluster}>
											{cluster.map((item) => controls.control(item, false, scope))}
										</div>
									))}
								</div>
							))}
						</div>
					) : (
						columns(group.items ?? [], perColumn).map((col, i) => (
							<div key={i} className={styles.column}>
								{col.map((item) => controls.control(item, showLabels, scope))}
							</div>
						))
					)}
				</div>
				{density === 'comfortable' && !simplified && (
					<span className={styles.groupLabel}>{group.label}</span>
				)}
			</div>
		);
	};

	const shownGroups = groups.slice(0, groups.length - fit.more);
	const underMore = groups.slice(groups.length - fit.more);

	return (
		<div
			role="group"
			aria-label={labels.ribbon}
			className={styles.ribbon}
			data-ribbon=""
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
					{props.end}
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
					onFocus={onPanelFocus}
				>
					<div role="toolbar" aria-label={labels.toolbar} className={styles.toolbar}>
						{shownGroups.map((group) => renderGroup(group, fit.levels[group.id]))}
						{underMore.length > 0 && (
							<div role="group" aria-label={labels.more} className={styles.group} data-level="more">
								{dropButton(
									'more',
									labels.more,
									MORE_GLYPH,
									underMore.map((g) => g.id),
									false,
								)}
							</div>
						)}
					</div>
				</div>
			)}
			{popup && (
				<div
					ref={popupRef}
					role="group"
					aria-label={popup.label}
					className={styles.popup}
					style={
						{
							'--folio-ribbon-popup-x': `${popup.x}px`,
							'--folio-ribbon-popup-y': `${popup.y}px`,
						} as CSSProperties
					}
				>
					{groups.filter((g) => popup.ids.includes(g.id)).map((g) => renderGroup(g))}
				</div>
			)}
			{controls.overlays}
		</div>
	);
}
