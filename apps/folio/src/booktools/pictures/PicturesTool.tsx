// Pictures in this book: every picture, how sharp it is where it is set, and a way to swap it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { PictureRow } from '@liminal-hq/folio-protocol/generated/PictureRow';
import { shelfLabels } from '../../books/shelfLabels';
import { useBooks } from '../../books/useBooks';
import { useAssetUrls } from '../../canvas/useAssetUrls';
import { t } from '../../i18n/messages';
import type { ToolProps } from '../tools';
import { useToolReport } from '../useToolReport';
import styles from './pictures.module.css';

export function PicturesTool({ client, id, book, revision, expert, apply, say }: ToolProps) {
	const { dialogs } = useBooks();
	const report = useToolReport(client, id, { tool: 'pictures' }, revision);
	const rows = report?.rows ?? [];
	const urls = useAssetUrls(
		client,
		id,
		rows.flatMap((row) => (row.asset ? [row.asset] : [])),
	);
	const labels = shelfLabels(book);
	const where = (row: PictureRow) => {
		const at = book.spreads.findIndex((s) => s.id === row.spread);
		return at >= 0 ? (labels[at] ?? '') : '';
	};
	const replace = async (row: PictureRow) => {
		const picked = await dialogs.pickImage();
		if (picked === null) return;
		try {
			const asset = await client.importImage(id, picked);
			await apply({ type: 'SetImage', frame: row.frame, asset, fit: null });
			say(t('tools.pictures.replaced', { where: where(row) }));
		} catch {
			say(t('tools.pictures.notReplaced'));
		}
	};

	if (report === null) return <p className={styles.note}>{t('tools.working')}</p>;
	if (rows.length === 0) return <p className={styles.note}>{t('tools.pictures.none')}</p>;
	const soft = rows.filter((r) => r.status === 'low' || r.status === 'veryLow').length;
	const lost = rows.filter((r) => r.status === 'missing' || r.status === 'unreadable').length;
	return (
		<div>
			<p className={styles.summary}>
				{t('tools.pictures.summary', { count: rows.length, soft, lost })}
			</p>
			<table className={styles.table}>
				<thead>
					<tr>
						<th scope="col">{t('tools.pictures.picture')}</th>
						<th scope="col">{t('tools.pictures.where')}</th>
						<th scope="col">{t('tools.pictures.sharpness')}</th>
						{expert && <th scope="col">{t('tools.pictures.size')}</th>}
						<td />
					</tr>
				</thead>
				<tbody>
					{rows.map((row) => (
						<tr key={row.frame} data-status={row.status}>
							<td>
								{row.asset && urls[row.asset] ? (
									<img className={styles.thumb} src={urls[row.asset]} alt={row.description ?? ''} />
								) : (
									<span className={styles.blank} aria-hidden="true" />
								)}
							</td>
							<td>{where(row)}</td>
							<td>
								<span className={styles.status} data-status={row.status}>
									{t(`tools.pictures.status.${row.status}`)}
								</span>
								{expert && row.ppi !== null && (
									<span className={styles.number}>
										{t('tools.pictures.ppi', { ppi: Math.round(row.ppi) })}
									</span>
								)}
							</td>
							{expert && (
								<td className={styles.number}>
									{row.width !== null && row.height !== null
										? `${row.width} × ${row.height} px · ${row.fit}`
										: ''}
								</td>
							)}
							<td>
								<button type="button" className={styles.button} onClick={() => void replace(row)}>
									{row.asset ? t('tools.pictures.replace') : t('tools.pictures.choose')}
								</button>
							</td>
						</tr>
					))}
				</tbody>
			</table>
		</div>
	);
}
