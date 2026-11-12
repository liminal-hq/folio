// What an export changed because a destination needs it, said after the files are made
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { t } from '../i18n/messages';
import styles from './ExportNotes.module.css';

export function ExportNotes({ notes }: { notes: string[] | undefined }) {
	if (!notes || notes.length === 0) return null;
	return (
		<section aria-label={t('export.notes')} className={styles.notes}>
			<p className={styles.title}>{t('export.notes')}</p>
			<ul className={styles.list}>
				{notes.map((note) => (
					<li key={note}>{note}</li>
				))}
			</ul>
		</section>
	);
}
