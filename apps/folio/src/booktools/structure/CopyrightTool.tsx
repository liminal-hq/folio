// Copyright page: the page behind the title page, made from what the book already says
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useState } from 'react';
import { t } from '../../i18n/messages';
import type { ToolProps } from '../tools';
import { useToolReport } from '../useToolReport';
import styles from './structure.module.css';

export function CopyrightTool({ client, id, book, revision, say }: ToolProps) {
	const [year, setYear] = useState(String(new Date().getFullYear()));
	const [holder, setHolder] = useState(book.authors[0]?.name ?? '');
	const [rights, setRights] = useState(t('tools.copyright.rightsDefault'));
	const [printing, setPrinting] = useState(t('tools.copyright.printingDefault'));
	const fields = {
		year: Number.parseInt(year, 10) || new Date().getFullYear(),
		holder,
		rights,
		printing,
	};
	const report = useToolReport(client, id, { tool: 'copyright', fields }, revision);
	const make = async () => {
		try {
			await client.bookToolFix(id, { tool: 'copyrightPage', fields });
			say(report?.present ? t('tools.copyright.updated') : t('tools.copyright.added'));
		} catch {
			say(t('tools.copyright.notMade'));
		}
	};
	return (
		<div className={styles.two}>
			<div className={styles.form}>
				<label className={styles.field}>
					<span>{t('tools.copyright.year')}</span>
					<input type="number" value={year} onChange={(event) => setYear(event.target.value)} />
				</label>
				<label className={styles.field}>
					<span>{t('tools.copyright.holder')}</span>
					<input type="text" value={holder} onChange={(event) => setHolder(event.target.value)} />
				</label>
				<label className={styles.field}>
					<span>{t('tools.copyright.rights')}</span>
					<input type="text" value={rights} onChange={(event) => setRights(event.target.value)} />
				</label>
				<label className={styles.field}>
					<span>{t('tools.copyright.printing')}</span>
					<input
						type="text"
						value={printing}
						onChange={(event) => setPrinting(event.target.value)}
					/>
				</label>
				<button type="button" className={styles.primary} onClick={() => void make()}>
					{report?.present ? t('tools.copyright.update') : t('tools.copyright.add')}
				</button>
				<p className={styles.hint}>{t('tools.copyright.where')}</p>
			</div>
			<div className={styles.page} aria-label={t('tools.copyright.preview')}>
				<pre className={styles.words}>{report?.text ?? ''}</pre>
			</div>
		</div>
	);
}
