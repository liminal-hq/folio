// The main window's screen: title bar, toolbar, work area, page shelf and status bar
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { MenuBar } from '@liminal-hq/folio-chrome/MenuBar/MenuBar';
import { WindowFrame } from '@liminal-hq/folio-chrome/WindowFrame';
import type { CSSProperties } from 'react';
import { useEffect, useState } from 'react';
import { APP_MENU_MNEMONICS } from '../commands/appMenuModel';
import { t } from '../i18n/messages';
import { tauriAppInfoClient, type AppInfoClient } from '../services/appInfoClient';
import { useSettings, useUpdateSettings } from '../settings/useSettings';
import { AppMenu } from './AppMenu';
import { AppTitleBar } from './AppTitleBar';
import styles from './MainScreen.module.css';
import { Segmented } from './Segmented';
import { useAppMenu } from './useAppMenu';

/** The app's version, or `null` until it is known (and for good outside Tauri). */
function useAppVersion(client: AppInfoClient): string | null {
	const [version, setVersion] = useState<string | null>(null);
	useEffect(() => {
		let active = true;
		client
			.appInfo()
			.then((info) => {
				if (active) setVersion(info.version);
			})
			.catch(() => {
				// Not in Tauri (a plain browser, or a test).
			});
		return () => {
			active = false;
		};
	}, [client]);
	return version;
}

export function MainScreen({ client = tauriAppInfoClient }: { client?: AppInfoClient }) {
	const settings = useSettings();
	const update = useUpdateSettings();
	const menu = useAppMenu();
	const version = useAppVersion(client);
	const menuBar = settings.interface.menuBar;
	const inspectorWidth = { '--folio-inspector-width': `${settings.interface.sidebarWidth}px` };

	return (
		<WindowFrame>
			<div className={styles.screen}>
				<AppTitleBar
					title={t('window.main.title')}
					start={<AppMenu items={menu.items} onSelect={menu.onSelect} menuBar={menuBar} />}
				/>
				{menuBar && (
					<MenuBar
						items={menu.barItems}
						onSelect={menu.onSelect}
						label={t('menu.bar')}
						moreLabel={t('menu.more')}
						mnemonics={APP_MENU_MNEMONICS}
					/>
				)}
				<div className={styles.toolbar} role="toolbar" aria-label={t('region.toolbar')}>
					<p className={styles.toolbarNote}>
						{settings.interface.toolbar === 'ribbon'
							? t('region.toolbar.ribbon')
							: t('region.toolbar.classic')}
					</p>
					<div className={styles.mode}>
						<span>{t('mode.label')}</span>
						<Segmented
							label={t('mode.label')}
							value={settings.mode}
							options={[
								{ value: 'easy', label: t('mode.easy') },
								{ value: 'expert', label: t('mode.expert') },
							]}
							onChange={(mode) => void update({ mode })}
						/>
					</div>
				</div>
				<main className={styles.body} style={inspectorWidth as CSSProperties}>
					<div className={styles.stageHeader}>
						<h1 className={styles.stageTitle}>{t('region.stage')}</h1>
					</div>
					<div className={styles.canvas}>
						<section className={styles.paper} aria-label={t('region.stage')}>
							<p className={styles.muted}>{t('region.stage.empty')}</p>
						</section>
					</div>
					<aside className={styles.inspector} aria-label={t('region.inspector')}>
						<h2 className={styles.inspectorTitle}>{t('region.inspector')}</h2>
						<p className={styles.muted}>{t('region.inspector.empty')}</p>
					</aside>
				</main>
				<section className={styles.shelf} aria-label={t('region.shelf')}>
					<p className={styles.muted}>{t('region.shelf.empty')}</p>
				</section>
				<footer className={styles.status} aria-label={t('status.bar')}>
					<span>{t('status.noBook')}</span>
					<span className={styles.statusGroup}>
						<span>{t('status.saved')}</span>
						{settings.jax !== 'off' && <span>{t('status.jax')}</span>}
						{version !== null && <span>{`${t('app.name')} ${version}`}</span>}
					</span>
				</footer>
			</div>
		</WindowFrame>
	);
}
