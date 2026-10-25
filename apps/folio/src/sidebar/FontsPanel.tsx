// The Fonts panel: the typefaces the book uses, where each is from, and whether the book keeps it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FontUse } from '@liminal-hq/folio-protocol/generated/FontUse';
import { t, type MessageKey } from '../i18n/messages';
import styles from './FontsPanel.module.css';

interface FontsPanelProps {
	/** `undefined` until the plugin has answered. */
	uses: FontUse[] | undefined;
	onKeep(family: string): void;
	onLetGo(family: string): void;
}

const ORIGIN: Record<FontUse['origin'], MessageKey> = {
	bundled: 'fonts.origin.bundled',
	book: 'fonts.origin.book',
	system: 'fonts.origin.system',
};

const LICENCE: Record<FontUse['licence'], MessageKey | null> = {
	free: null,
	printOnly: 'fonts.licence.printOnly',
	restricted: 'fonts.licence.restricted',
};

export function FontsPanel({ uses, onKeep, onLetGo }: FontsPanelProps) {
	if (uses === undefined) return <p className={styles.note}>{t('fonts.loading')}</p>;
	return (
		<div className={styles.panel}>
			<p className={styles.intro}>{t('fonts.intro')}</p>
			<ul className={styles.list}>
				{uses.map((use) => {
					const licence = LICENCE[use.licence];
					return (
						<li key={use.family} className={styles.row} data-missing={use.missing || undefined}>
							<span className={styles.text}>
								<span className={styles.name}>{use.family}</span>
								<span className={styles.status}>
									{use.missing ? t('fonts.missing') : t(ORIGIN[use.origin])}
								</span>
								{licence && !use.missing && <span className={styles.licence}>{t(licence)}</span>}
							</span>
							{use.origin === 'system' && !use.missing && use.licence === 'free' && (
								<button type="button" className={styles.action} onClick={() => onKeep(use.family)}>
									{t('fonts.keep')}
								</button>
							)}
							{use.origin === 'book' && (
								<button type="button" className={styles.action} onClick={() => onLetGo(use.family)}>
									{t('fonts.letGo')}
								</button>
							)}
						</li>
					);
				})}
			</ul>
		</div>
	);
}
