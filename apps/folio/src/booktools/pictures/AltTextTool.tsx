// Alt text: what each picture shows, in words, for a reader who cannot see it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { PictureRow } from '@liminal-hq/folio-protocol/generated/PictureRow';
import { useEffect, useState } from 'react';
import { shelfLabels } from '../../books/shelfLabels';
import { useAssetUrls } from '../../canvas/useAssetUrls';
import { t } from '../../i18n/messages';
import type { ToolProps } from '../tools';
import { useToolReport } from '../useToolReport';
import styles from './pictures.module.css';

/** One picture and the box for what it shows; kept when the box is left, as one undo step. */
function Described({
	row,
	url,
	where,
	onKeep,
	onDecorative,
}: {
	row: PictureRow;
	url: string | undefined;
	where: string;
	onKeep(text: string): void;
	onDecorative(decorative: boolean): void;
}) {
	const [draft, setDraft] = useState(row.description ?? '');
	useEffect(() => setDraft(row.description ?? ''), [row.description]);
	return (
		<li className={styles.alt}>
			{url ? (
				<img className={styles.thumb} src={url} alt="" />
			) : (
				<span className={styles.blank} aria-hidden="true" />
			)}
			<div className={styles.altField}>
				<label className={styles.altLabel}>
					<span>{t('tools.alt.label', { where })}</span>
					<textarea
						rows={2}
						value={draft}
						maxLength={500}
						onChange={(event) => setDraft(event.target.value)}
						onBlur={() => draft.trim() !== (row.description ?? '').trim() && onKeep(draft.trim())}
					/>
				</label>
				<label className={styles.altDecorative}>
					<input
						type="checkbox"
						checked={row.decorative}
						onChange={(event) => onDecorative(event.target.checked)}
					/>
					{t('tools.alt.decorative')}
				</label>
				{row.automatic && <span className={styles.note}>{t('tools.alt.automatic')}</span>}
			</div>
		</li>
	);
}

export function AltTextTool({ client, id, book, revision, apply, say }: ToolProps) {
	const report = useToolReport(client, id, { tool: 'pictures' }, revision);
	const rows = report?.rows ?? [];
	const urls = useAssetUrls(
		client,
		id,
		rows.flatMap((row) => (row.asset ? [row.asset] : [])),
	);
	const labels = shelfLabels(book);
	if (report === null) return <p className={styles.note}>{t('tools.working')}</p>;
	if (rows.length === 0) return <p className={styles.note}>{t('tools.pictures.none')}</p>;
	const described = rows.filter(
		(row) => (row.description ?? '').trim() !== '' || row.decorative || row.automatic,
	).length;
	return (
		<div>
			<p className={styles.summary}>{t('tools.alt.summary', { described, count: rows.length })}</p>
			<ul className={styles.altList}>
				{rows.map((row) => (
					<Described
						key={row.frame}
						row={row}
						url={row.asset ? urls[row.asset] : undefined}
						where={labels[book.spreads.findIndex((s) => s.id === row.spread)] ?? ''}
						onDecorative={(decorative) => {
							void apply({
								type: 'SetFrameDecorative',
								on: { spread: row.spread },
								frame: row.frame,
								decorative,
							});
							say(decorative ? t('tools.alt.markedDecorative') : t('tools.alt.unmarkedDecorative'));
						}}
						onKeep={(text) => {
							void apply({
								type: 'SetFrameDescription',
								on: { spread: row.spread },
								frame: row.frame,
								description: text === '' ? null : text,
							});
							say(text === '' ? t('tools.alt.cleared') : t('tools.alt.kept'));
						}}
					/>
				))}
			</ul>
		</div>
	);
}
