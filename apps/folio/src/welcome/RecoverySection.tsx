// The start screen's offer of unsaved work a crash left behind
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Recoverable } from '@liminal-hq/folio-plugin-books';
import { useCallback, useEffect, useState } from 'react';
import { useBooks } from '../books/useBooks';
import { t } from '../i18n/messages';
import { keptWhen } from './recovered';
import { recentEntry } from './recent';
import styles from './WelcomeScreen.module.css';

/** What a crash left, newest first, each with a way to bring it back or to give it up. */
export function RecoverySection() {
	const { client, actions } = useBooks();
	const [left, setLeft] = useState<Recoverable[]>([]);
	const read = useCallback(() => {
		let live = true;
		client
			.recoverable()
			.then((found) => live && setLeft(found))
			.catch(() => live && setLeft([]));
		return () => {
			live = false;
		};
	}, [client]);
	useEffect(() => read(), [read]);

	if (left.length === 0) return null;
	return (
		<section aria-labelledby="welcome-recovery">
			<h2 id="welcome-recovery" className={styles.sideHeading}>
				{t('recovery.heading')}
			</h2>
			<ul className={styles.list}>
				{left.map((item) => (
					<li key={item.id} className={styles.recovered}>
						<span className={styles.rowName}>{item.title}</span>
						<span className={styles.rowFolder}>
							{item.path === null ? t('recovery.never') : recentEntry(item.path).folder} ·{' '}
							{t('recovery.saved', { when: keptWhen(item.savedAt) })}
						</span>
						<span className={styles.recoveredActions}>
							<button
								type="button"
								className={styles.action}
								aria-label={t('recovery.restore.for', { title: item.title })}
								onClick={() => void actions.restore(item.id).then(read)}
							>
								{t('recovery.restore')}
							</button>
							<button
								type="button"
								className={styles.action}
								aria-label={t('recovery.discard.for', { title: item.title })}
								onClick={() => void actions.discard(item.id).then(read)}
							>
								{t('recovery.discard')}
							</button>
						</span>
					</li>
				))}
			</ul>
		</section>
	);
}
