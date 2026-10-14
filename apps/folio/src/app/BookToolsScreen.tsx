// The Book tools window: the utility window that hosts one workbench at a time
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { WindowFrame } from '@liminal-hq/folio-chrome/WindowFrame';
import { t } from '../i18n/messages';
import { AppTitleBar } from './AppTitleBar';
import styles from './BookToolsScreen.module.css';

export function BookToolsScreen() {
	return (
		<WindowFrame>
			<div className={styles.screen}>
				<AppTitleBar title={t('window.bookTools.title')} />
				<main className={styles.body}>
					<h1 className={styles.heading}>{t('bookTools.heading')}</h1>
					<p className={styles.note}>{t('bookTools.empty')}</p>
				</main>
			</div>
		</WindowFrame>
	);
}
