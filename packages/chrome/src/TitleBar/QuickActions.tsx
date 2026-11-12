// A strip of icon buttons for the title bar's start side: quick shortcuts such as Save and New
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { forwardRef, useRef, type KeyboardEvent, type ReactNode } from 'react';
import '../tokens.css';
import styles from './QuickActions.module.css';

/** One shortcut in the strip. The chrome knows nothing about what it does. */
export interface QuickAction {
	id: string;
	/** Accessible name and the first part of the tooltip. */
	label: string;
	/** The button's glyph; it is hidden from assistive technology. */
	icon: ReactNode;
	/** Shown in the tooltip after the label, as `Label (Ctrl+S)`. */
	shortcut?: string;
	/** Drawn dimmed and does nothing, but stays reachable by keyboard so it can be discovered. */
	disabled?: boolean;
	/** For a toggle: whether it is on. Omit for a plain button. */
	pressed?: boolean;
	/** Draws a dot on the icon, for example to say there is something unsaved. */
	badge?: boolean;
	/** Said after the label when `badge` is set, for assistive technology and the tooltip. */
	badgeLabel?: string;
	onSelect: () => void;
}

export interface QuickActionsProps {
	actions: readonly QuickAction[];
	/** Accessible name of the toolbar. */
	label: string;
	/** Hides the strip because there is no room for it; the title bar decides. */
	collapsed?: boolean;
}

/** The tooltip text for an action: its name, what its badge says, and its shortcut. */
export function quickActionTitle(action: QuickAction): string {
	const name =
		action.badge && action.badgeLabel ? `${action.label}, ${action.badgeLabel}` : action.label;
	return action.shortcut ? `${name} (${action.shortcut})` : name;
}

/**
 * An ARIA toolbar with one tab stop. Left and Right (or Up and Down) move between the buttons,
 * Home and End jump to the ends, and Escape hands focus back to wherever it came from.
 */
export const QuickActions = forwardRef<HTMLDivElement, QuickActionsProps>(function QuickActions(
	{ actions, label, collapsed = false },
	forwardedRef,
) {
	const toolbar = useRef<HTMLDivElement | null>(null);
	// Where focus was before it entered the toolbar, so Escape can return it.
	const cameFrom = useRef<HTMLElement | null>(null);
	// The button that holds the tab stop; null means the first.
	const current = useRef<string | null>(null);

	const buttons = () => Array.from(toolbar.current?.querySelectorAll('button') ?? []);

	const setRef = (node: HTMLDivElement | null) => {
		toolbar.current = node;
		if (typeof forwardedRef === 'function') forwardedRef(node);
		else if (forwardedRef) forwardedRef.current = node;
	};

	const move = (to: HTMLButtonElement | undefined) => {
		if (!to) return;
		current.current = to.dataset.quickAction ?? null;
		for (const button of buttons()) button.tabIndex = button === to ? 0 : -1;
		to.focus();
	};

	const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
		if (event.altKey || event.ctrlKey || event.metaKey) return;
		const all = buttons();
		const at = all.indexOf(document.activeElement as HTMLButtonElement);
		switch (event.key) {
			case 'ArrowRight':
			case 'ArrowDown':
				event.preventDefault();
				move(all[(at + 1) % all.length]);
				break;
			case 'ArrowLeft':
			case 'ArrowUp':
				event.preventDefault();
				move(all[(at - 1 + all.length) % all.length]);
				break;
			case 'Home':
				event.preventDefault();
				move(all[0]);
				break;
			case 'End':
				event.preventDefault();
				move(all[all.length - 1]);
				break;
			case 'Escape': {
				const back = cameFrom.current;
				event.preventDefault();
				event.stopPropagation();
				if (back?.isConnected) back.focus();
				else (document.activeElement as HTMLElement | null)?.blur();
				break;
			}
			default:
				break;
		}
	};

	if (actions.length === 0) return null;
	const tabStop = actions.some((a) => a.id === current.current) ? current.current : actions[0]?.id;

	return (
		<div
			ref={setRef}
			className={styles.strip}
			// The strip itself is passive space, so the gaps between buttons still drag the window.
			data-tauri-drag-region=""
			data-quick-actions=""
			role="toolbar"
			aria-label={label}
			aria-orientation="horizontal"
			hidden={collapsed}
			data-collapsed={collapsed || undefined}
			onKeyDown={onKeyDown}
			onFocus={(event) => {
				const from = event.relatedTarget;
				if (from instanceof HTMLElement && !toolbar.current?.contains(from))
					cameFrom.current = from;
			}}
		>
			{actions.map((action) => {
				const title = quickActionTitle(action);
				return (
					<button
						key={action.id}
						type="button"
						className={styles.button}
						data-quick-action={action.id}
						aria-label={title}
						title={title}
						aria-disabled={action.disabled || undefined}
						aria-pressed={action.pressed}
						tabIndex={action.id === tabStop ? 0 : -1}
						onFocus={() => {
							current.current = action.id;
							for (const button of buttons()) {
								button.tabIndex = button.dataset.quickAction === action.id ? 0 : -1;
							}
						}}
						onClick={() => {
							if (!action.disabled) action.onSelect();
						}}
					>
						<span className={styles.icon} aria-hidden="true">
							{action.icon}
						</span>
						{action.badge ? <span className={styles.badge} aria-hidden="true" /> : null}
					</button>
				);
			})}
		</div>
	);
});
