// Entry point: picks the screen for this webview's window kind
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import type { WindowKind } from '@liminal-hq/folio-protocol/generated/WindowKind';
import { AppWindowChrome } from './app/AppWindowChrome';
import { AboutScreen } from './app/AboutScreen';
import { BookToolsScreen } from './app/BookToolsScreen';
import { MainScreen } from './app/MainScreen';
import { NewBookWizard } from './wizard/NewBookWizard';
import { BooksProvider } from './books/BooksProvider';
import { tauriBooksClient } from './books/booksClient';
import { tauriFileDialogs } from './books/dialogs';
import { PreferencesScreen } from './app/PreferencesScreen';
import { windowKindFromLabel } from './app/windowKind';
import { SettingsProvider } from './settings/SettingsProvider';
import { tauriSettingsClient } from './settings/settingsClient';
import { initLogger } from './services/logger';
import { applyPlatform } from './theme/platform';
import { ThemeRoot } from './theme/ThemeRoot';
import './theme/tokens.css';

// A plain switch on the window kind: each webview hosts exactly one screen, so there is no
// in-window navigation to route.
function screenFor(kind: WindowKind | null) {
	switch (kind) {
		case 'Main':
			return (
				<BooksProvider client={tauriBooksClient} dialogs={tauriFileDialogs}>
					<MainScreen />
				</BooksProvider>
			);
		case 'About':
			return <AboutScreen />;
		case 'Preferences':
			return <PreferencesScreen />;
		case 'BookTools':
			return <BookToolsScreen />;
		case 'WizardNew':
			return <NewBookWizard />;
		case null:
			return null;
	}
}

const label = getCurrentWebviewWindow().label;

initLogger(label);
applyPlatform();

createRoot(document.getElementById('root')!).render(
	<StrictMode>
		<SettingsProvider client={tauriSettingsClient}>
			<ThemeRoot>
				<AppWindowChrome>{screenFor(windowKindFromLabel(label))}</AppWindowChrome>
			</ThemeRoot>
		</SettingsProvider>
	</StrictMode>,
);
