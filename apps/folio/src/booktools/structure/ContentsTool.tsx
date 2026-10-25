// Contents: the chapters of the story with the page each begins on, set on a page of their own
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { t } from '../../i18n/messages';
import type { ToolProps } from '../tools';
import { useToolReport } from '../useToolReport';
import styles from './structure.module.css';

export function ContentsTool({ client, id, revision, say }: ToolProps) {
	const report = useToolReport(client, id, { tool: 'contents' }, revision);
	if (report === null) return <p className={styles.note}>{t('tools.working')}</p>;
	if (report.entries.length === 0) return <p className={styles.note}>{t('tools.contents.none')}</p>;
	const make = async () => {
		try {
			await client.bookToolFix(id, { tool: 'contentsPage' });
			say(report.present ? t('tools.contents.updated') : t('tools.contents.added'));
		} catch {
			say(t('tools.contents.notMade'));
		}
	};
	return (
		<div className={styles.two}>
			<div className={styles.form}>
				<button type="button" className={styles.primary} onClick={() => void make()}>
					{report.present ? t('tools.contents.update') : t('tools.contents.add')}
				</button>
				<p className={styles.hint}>{t('tools.contents.where')}</p>
			</div>
			<ol className={styles.toc} aria-label={t('tools.contents.preview')}>
				{report.entries.map((entry, index) => (
					<li key={index} className={styles.entry}>
						<span>{entry.title}</span>
						<span className={styles.leader} aria-hidden="true" />
						<span>{entry.page}</span>
					</li>
				))}
			</ol>
		</div>
	);
}
