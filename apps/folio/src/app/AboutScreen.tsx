// The About window: the mark, the name, the version and a way to copy the build details
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { WindowFrame } from '@liminal-hq/folio-chrome/WindowFrame';
import { useEffect, useState } from 'react';
import type { AppInfo } from '@liminal-hq/folio-protocol/generated/AppInfo';
import { t } from '../i18n/messages';
import { AppMarkIcon } from '../icons/AppIcons';
import { tauriAppInfoClient, type AppInfoClient } from '../services/appInfoClient';
import styles from './AboutScreen.module.css';
import { AppTitleBar } from './AppTitleBar';

interface AboutScreenProps {
	client?: AppInfoClient;
	/** Puts text on the clipboard; the webview's by default. */
	copy?: (text: string) => Promise<void>;
}

export function AboutScreen({
	client = tauriAppInfoClient,
	copy = (text) => navigator.clipboard.writeText(text),
}: AboutScreenProps) {
	const [info, setInfo] = useState<AppInfo | null>(null);
	const [copied, setCopied] = useState(false);

	useEffect(() => {
		let active = true;
		client
			.appInfo()
			.then((value) => {
				if (active) setInfo(value);
			})
			.catch(() => {
				// Not in Tauri: the window shows the name and tagline without the build details.
			});
		return () => {
			active = false;
		};
	}, [client]);

	const copyDetails = () => {
		if (info === null) return;
		const details = t('about.details', {
			version: info.version,
			identifier: info.identifier,
			platform: info.platform,
		});
		copy(details)
			.then(() => setCopied(true))
			.catch(() => setCopied(false));
	};

	return (
		<WindowFrame>
			<div className={styles.screen}>
				<AppTitleBar title={t('window.about.title')} closeOnly />
				<main className={styles.body}>
					<div className={styles.mark}>
						<AppMarkIcon size={128} />
					</div>
					<h1 className={styles.name}>{t('app.name')}</h1>
					<p className={styles.tagline}>{t('about.tagline')}</p>
					{info !== null && (
						<>
							<p className={styles.fact}>{t('about.version', { version: info.version })}</p>
							<p className={styles.fact}>
								{t('about.identifier', { identifier: info.identifier, platform: info.platform })}
							</p>
						</>
					)}
					<p className={styles.fact}>{t('about.licence')}</p>
					<p className={styles.fact}>{t('about.copyright')}</p>
					{info !== null && (
						<button type="button" className={styles.copy} onClick={copyDetails}>
							{copied ? t('about.copied') : t('about.copyDetails')}
						</button>
					)}
				</main>
			</div>
		</WindowFrame>
	);
}
