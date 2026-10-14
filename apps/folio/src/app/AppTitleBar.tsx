// The app's title bar: the shared chrome configured from the OS's own preferences
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { TitleBar } from '@liminal-hq/folio-chrome/TitleBar';
import { TitleBarTitle } from '@liminal-hq/folio-chrome/TitleBar/TitleBarTitle';
import type { ReactNode } from 'react';
import { chromeLabels } from '../i18n/chromeLabels';
import { useTitlebarPreferences } from '../services/titlebarPreferences';
import { titlebarConfigFor } from './titlebarConfig';
import { useWindowCapabilities } from './windowCapabilities';

interface AppTitleBarProps {
	title: string;
	/** The start slot, after any start-side window buttons: the main window puts its app menu here. */
	start?: ReactNode;
}

export function AppTitleBar({ title, start }: AppTitleBarProps) {
	const preferences = useTitlebarPreferences();
	const capabilities = useWindowCapabilities();
	const config = titlebarConfigFor(preferences, document.documentElement.dataset.platform);
	return (
		<TitleBar
			start={start}
			center={<TitleBarTitle>{title}</TitleBarTitle>}
			controlsStyle={config.controlsStyle}
			buttonLayout={config.buttonLayout}
			titlebarActions={config.titlebarActions}
			titleAlign={config.titleAlign}
			labels={chromeLabels()}
			// Wayland has no protocol for staying on top, so the pin would do nothing there; the
			// compositor's own menu ("More options…") has a working one.
			showAlwaysOnTop={capabilities?.alwaysOnTop === true}
		/>
	);
}
