// The main window's screen: the title bar over the work area, which the editor will fill
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { WindowFrame } from '@liminal-hq/folio-chrome/WindowFrame';
import { useEffect, useState } from 'react';
import type { AppInfo } from '@liminal-hq/folio-protocol/generated/AppInfo';
import { t } from '../i18n/messages';
import { tauriAppInfoClient, type AppInfoClient } from '../services/appInfoClient';
import { AppTitleBar } from './AppTitleBar';
import styles from './MainScreen.module.css';

type Load = { state: 'loading' } | { state: 'ready'; info: AppInfo } | { state: 'unavailable' };

export function MainScreen({ client = tauriAppInfoClient }: { client?: AppInfoClient }) {
	const [load, setLoad] = useState<Load>({ state: 'loading' });

	useEffect(() => {
		let active = true;
		client
			.appInfo()
			.then((info) => {
				if (active) setLoad({ state: 'ready', info });
			})
			.catch(() => {
				// Not in Tauri (a plain browser, or a test).
				if (active) setLoad({ state: 'unavailable' });
			});
		return () => {
			active = false;
		};
	}, [client]);

	return (
		<WindowFrame>
			<div className={styles.screen}>
				<AppTitleBar title={t('window.main.title')} />
				<main className={styles.body}>
					{load.state === 'loading' && <p className={styles.headline}>{t('main.loading')}</p>}
					{load.state === 'unavailable' && (
						<p className={styles.headline}>{t('main.unavailable')}</p>
					)}
					{load.state === 'ready' && (
						<>
							<p className={styles.headline}>
								{t('main.running', { version: load.info.version, platform: load.info.platform })}
							</p>
							<p className={styles.detail}>
								{t('main.identifier', { identifier: load.info.identifier })}
							</p>
						</>
					)}
				</main>
			</div>
		</WindowFrame>
	);
}
