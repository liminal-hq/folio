// Window frame: clips a frameless window to rounded corners, square when maximised
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useSyncExternalStore, type ReactNode } from 'react';
import { isDimmed, subscribeDimming } from '../Dialog/dimming';
import { useWindowFocused, useWindowMaximised } from '../WindowChromeProvider/WindowChromeProvider';
import '../tokens.css';
import styles from './WindowFrame.module.css';

export interface WindowFrameProps {
	children: ReactNode;
	className?: string;
}

/**
 * The outermost element of a frameless, transparent window. Its surface is rounded
 * and clipped to `--folio-window-radius`, given a hairline border, and shadowed with
 * `--folio-window-shadow` inside a transparent margin of `--folio-window-shadow-margin`.
 * While the window is unfocused it uses `--folio-window-shadow-unfocused` instead, falling
 * back to the focused shadow. The margin and shadow default to none, for platforms where
 * the OS draws its own. While a dialog is open the surface dims itself (a dialog's own backdrop
 * would also dim the shadow margin and show the square window behind the rounded one).
 * When the window is maximised the margin, shadow, rounding and border all drop so
 * it meets the screen edges cleanly. `className` applies to the visible surface. Window state comes
 * from `WindowChromeProvider`, which must be an ancestor.
 */
export function WindowFrame({ children, className }: WindowFrameProps) {
	const maximised = useWindowMaximised();
	const focused = useWindowFocused();
	const dimmed = useSyncExternalStore(subscribeDimming, isDimmed, () => false);
	// Tells dialogs a frame dims the window, so they leave their own backdrop clear.
	useEffect(() => {
		const root = globalThis.document?.documentElement;
		if (!root) return;
		root.dataset.framedWindow = 'true';
		return () => {
			delete root.dataset.framedWindow;
		};
	}, []);
	const surface = [styles.surface, className ?? ''].filter(Boolean).join(' ');
	return (
		<div
			className={styles.frame}
			data-maximised={maximised}
			data-focused={focused}
			data-dimmed={dimmed}
		>
			<div className={surface}>{children}</div>
		</div>
	);
}
