// Entry point: picks the screen for this webview's window kind
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import type { WindowKind } from '@liminal-hq/folio-protocol/generated/WindowKind';
import { AppWindowChrome } from './app/AppWindowChrome';
import { MainScreen } from './app/MainScreen';
import { windowKindFromLabel } from './app/windowKind';
import { initLogger } from './services/logger';
import { applyPlatform } from './theme/platform';
import './theme/tokens.css';

// A plain switch on the window kind: each webview hosts exactly one screen, so there is no
// in-window navigation to route.
function screenFor(kind: WindowKind | null) {
	switch (kind) {
		case 'Main':
			return <MainScreen />;
		// The child windows' screens arrive with the windows themselves.
		case 'About':
		case 'Preferences':
		case 'BookTools':
		case null:
			return null;
	}
}

const label = getCurrentWebviewWindow().label;

initLogger(label);
applyPlatform();

createRoot(document.getElementById('root')!).render(
	<StrictMode>
		<AppWindowChrome>{screenFor(windowKindFromLabel(label))}</AppWindowChrome>
	</StrictMode>,
);
