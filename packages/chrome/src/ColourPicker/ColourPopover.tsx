// A floating home for the picker: placed by its opener, kept in the window, closed by Escape or a click away
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useLayoutEffect, useRef, useState, type ReactNode } from 'react';
import { createPortal } from 'react-dom';
import '../tokens.css';
import styles from './ColourPicker.module.css';

export interface ColourPopoverProps {
	/** The control that opened it; the popover sits under it, or over it when there is no room below. */
	anchor: HTMLElement;
	/** Accessible name of the dialog. */
	label: string;
	/** Asked to close: `escape` returns focus to the opener when it was inside, `outside` leaves it. */
	onClose(reason: 'escape' | 'outside'): void;
	children: ReactNode;
}

const MARGIN = 8;

/**
 * Marked `data-keeps-editor` so an app's text editor does not end when focus moves in here, and
 * `data-folio-menu-root` so a ribbon drop-down it was opened from stays open under it.
 */
export function ColourPopover({ anchor, label, onClose, children }: ColourPopoverProps) {
	const root = useRef<HTMLDivElement>(null);
	const [at, setAt] = useState<{ left: number; top: number } | null>(null);
	const close = useRef(onClose);
	close.current = onClose;

	useLayoutEffect(() => {
		const el = root.current;
		if (!el) return;
		const place = () => {
			const rect = anchor.getBoundingClientRect();
			const width = el.offsetWidth;
			const height = el.offsetHeight;
			const left = Math.max(MARGIN, Math.min(rect.left, window.innerWidth - width - MARGIN));
			const below = rect.bottom + 4;
			const top =
				below + height > window.innerHeight - MARGIN && rect.top - height - 4 > MARGIN
					? rect.top - height - 4
					: Math.max(MARGIN, Math.min(below, window.innerHeight - height - MARGIN));
			setAt({ left, top });
		};
		place();
		window.addEventListener('resize', place);
		const observer = typeof ResizeObserver === 'undefined' ? null : new ResizeObserver(place);
		observer?.observe(el);
		return () => {
			window.removeEventListener('resize', place);
			observer?.disconnect();
		};
	}, [anchor]);

	useLayoutEffect(() => {
		const away = (event: Event) => {
			const target = event.target;
			if (!(target instanceof Node)) return;
			if (root.current?.contains(target) || anchor.contains(target)) return;
			close.current('outside');
		};
		const escape = (event: KeyboardEvent) => {
			if (event.key !== 'Escape') return;
			event.preventDefault();
			event.stopPropagation();
			const inside = root.current?.contains(document.activeElement) ?? false;
			close.current('escape');
			if (inside) anchor.focus();
		};
		document.addEventListener('pointerdown', away, true);
		window.addEventListener('keydown', escape, true);
		return () => {
			document.removeEventListener('pointerdown', away, true);
			window.removeEventListener('keydown', escape, true);
		};
	}, [anchor]);

	return createPortal(
		<div
			ref={root}
			className={styles.popover}
			role="dialog"
			aria-label={label}
			data-keeps-editor=""
			data-folio-menu-root=""
			style={{ left: at?.left ?? 0, top: at?.top ?? 0, visibility: at ? 'visible' : 'hidden' }}
		>
			{children}
		</div>,
		// Inside a modal dialog it must be in the dialog, or the page's top layer hides it and makes it inert.
		anchor.closest('dialog') ?? document.body,
	);
}
