// The content of a dialog that is a window of its own: title bar, scrolling body and button row
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import {
	useCallback,
	useEffect,
	useId,
	useLayoutEffect,
	useMemo,
	useRef,
	type KeyboardEvent as ReactKeyboardEvent,
	type MouseEvent as ReactMouseEvent,
	type ReactNode,
	type RefObject,
} from 'react';
import { DialogContext, type DialogCloseReason } from '../Dialog/dialogContext';
import { defaultFocusTarget, trapTab } from '../Dialog/focus';
import { WindowFrame } from '../WindowFrame/WindowFrame';
import '../tokens.css';
import styles from './DialogWindowFrame.module.css';

export interface DialogWindowFrameProps {
	/** The dialog's accessible name; the title bar should show the same words. */
	title: string;
	/** The title bar, drawn by the app so it follows the person's window-button preferences. */
	titleBar: ReactNode;
	/** Supporting text, wired up as the accessible description. */
	description?: ReactNode;
	children?: ReactNode;
	/** Buttons row; `DialogActions` is the usual content. */
	footer?: ReactNode;
	/** Asked for when the dialog wants to go away (Esc, or a button that closes). */
	onClose: (reason: DialogCloseReason) => void;
	/** Runs when Enter is pressed in a field: the dialog's default button. */
	onDefaultAction?: () => void;
	/** The element or CSS selector to focus on start. Defaults to the first field, then the primary button. */
	initialFocus?: RefObject<HTMLElement | null> | string;
	/** When false, Esc does nothing, for questions that must be answered. */
	dismissible?: boolean;
}

/** Fields where Enter means "go", not "new line" or "press this button". */
const ENTER_FIELDS = 'input:not([type="checkbox"]):not([type="radio"]):not([type="button"])';

/**
 * A form dialog is not maximised: Tauri's drag script maximises on the second press of a double
 * click, and the title bar's own handler does the same on `dblclick`. Both are stopped here, so
 * whichever title bar the app draws, double-clicking it does nothing.
 */
function stopDoubleClick(event: ReactMouseEvent<HTMLElement>) {
	if (event.detail < 2) return;
	event.preventDefault();
	event.stopPropagation();
}

/**
 * Lays out a dialog window. The description, body and buttons share one bounded, centred column
 * (`--folio-dialog-column`), so in a window made larger than its content the content stays
 * together in the middle and the buttons sit at the bottom of that column, not in a far corner.
 * The body scrolls and the buttons stay pinned at the bottom of the window.
 * It uses a `dialog` element held open (not modal: the window is the
 * modal) so the focus rules and the footer buttons behave exactly as they do in the page.
 */
export function DialogWindowFrame({
	title,
	titleBar,
	description,
	children,
	footer,
	onClose,
	onDefaultAction,
	initialFocus,
	dismissible = true,
}: DialogWindowFrameProps) {
	const dialogRef = useRef<HTMLDialogElement>(null);
	const surfaceRef = useRef<HTMLDivElement>(null);
	const titleId = useId();
	const descriptionId = useId();

	const live = useRef({ onClose, onDefaultAction, dismissible });
	useLayoutEffect(() => {
		live.current = { onClose, onDefaultAction, dismissible };
	});

	const requestClose = useCallback((reason: DialogCloseReason) => live.current.onClose(reason), []);
	const context = useMemo(() => ({ requestClose }), [requestClose]);

	// Focus starts in the dialog, and Esc answers unless a dialog nested inside is open.
	useEffect(() => {
		const dialog = dialogRef.current;
		const surface = surfaceRef.current;
		if (!dialog || !surface) return;
		let target: HTMLElement | null = null;
		if (typeof initialFocus === 'string') target = dialog.querySelector<HTMLElement>(initialFocus);
		else if (initialFocus) target = initialFocus.current;
		target ??= defaultFocusTarget(dialog);
		(target ?? surface).focus();

		const onKeyDown = (event: KeyboardEvent) => {
			if (event.key !== 'Escape' || event.defaultPrevented) return;
			if (dialog.querySelector('dialog[open]') !== null) return;
			event.preventDefault();
			if (live.current.dismissible) live.current.onClose('escape');
		};
		document.addEventListener('keydown', onKeyDown);
		return () => document.removeEventListener('keydown', onKeyDown);
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	const onKeyDown = (event: ReactKeyboardEvent<HTMLDialogElement>) => {
		const dialog = dialogRef.current;
		const surface = surfaceRef.current;
		if (!dialog || !surface) return;
		if (event.key === 'Tab') {
			event.stopPropagation();
			trapTab(event.nativeEvent, dialog, surface);
			return;
		}
		if (
			event.key === 'Enter' &&
			!event.defaultPrevented &&
			!event.nativeEvent.isComposing &&
			event.target instanceof HTMLElement &&
			event.target.matches(ENTER_FIELDS) &&
			live.current.onDefaultAction
		) {
			event.preventDefault();
			live.current.onDefaultAction();
		}
	};

	return (
		<WindowFrame>
			<DialogContext.Provider value={context}>
				<dialog
					open
					ref={dialogRef}
					className={styles.window}
					aria-labelledby={titleId}
					aria-describedby={description ? descriptionId : undefined}
					onKeyDown={onKeyDown}
				>
					<div
						className={styles.titleBar}
						onMouseDownCapture={stopDoubleClick}
						onMouseUpCapture={stopDoubleClick}
						onDoubleClickCapture={stopDoubleClick}
					>
						{titleBar}
					</div>
					<div ref={surfaceRef} className={styles.surface} tabIndex={-1}>
						<h1 id={titleId} className={styles.visuallyHidden}>
							{title}
						</h1>
						{description ? (
							<div className={styles.descriptionBar}>
								<p id={descriptionId} className={`${styles.description} ${styles.column}`}>
									{description}
								</p>
							</div>
						) : null}
						{children ? (
							<div className={styles.body} data-dialog-body>
								<div className={styles.column} data-dialog-column>
									{children}
								</div>
							</div>
						) : (
							<div className={styles.spacer} />
						)}
						{footer ? (
							<footer className={styles.footerBar} data-dialog-footer>
								<div className={styles.column}>{footer}</div>
							</footer>
						) : null}
					</div>
				</dialog>
			</DialogContext.Provider>
		</WindowFrame>
	);
}
