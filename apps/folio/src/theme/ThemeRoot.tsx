// Applies the colour scheme and control size the settings choose to the root element
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, type ReactNode } from 'react';
import { useSettings } from '../settings/useSettings';

/**
 * Sets `data-theme` (`light` or `dark`, absent to follow the system) and `data-larger-controls` on
 * the root element, which `tokens.css` reads. Book paper is never themed, so nothing here touches it.
 */
export function ThemeRoot({
	children,
	root = document.documentElement,
}: {
	children: ReactNode;
	root?: HTMLElement;
}) {
	const { colourMode, interface: ui } = useSettings();

	useEffect(() => {
		if (colourMode === 'system') delete root.dataset.theme;
		else root.dataset.theme = colourMode;
	}, [colourMode, root]);

	useEffect(() => {
		if (ui.largerControls) root.dataset.largerControls = 'true';
		else delete root.dataset.largerControls;
	}, [ui.largerControls, root]);

	return <>{children}</>;
}
