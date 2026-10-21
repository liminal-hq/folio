// Consistent styles: text boxes that set their words differently from the style they name
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { shelfLabels } from '../../books/shelfLabels';
import { t, type MessageKey } from '../../i18n/messages';
import type { ToolProps } from '../tools';
import { useToolReport } from '../useToolReport';
import styles from './typesetting.module.css';

const CHANGE: Record<string, MessageKey> = {
	size: 'tools.styles.change.size',
	align: 'tools.styles.change.align',
	bold: 'tools.styles.change.bold',
	italic: 'tools.styles.change.italic',
	font: 'tools.styles.change.font',
};

export function StylesTool({ client, id, book, revision, say }: ToolProps) {
	const report = useToolReport(client, id, { tool: 'styles' }, revision);
	const labels = shelfLabels(book);
	if (report === null) return <p className={styles.note}>{t('tools.working')}</p>;
	if (report.frames.length === 0) return <p className={styles.note}>{t('tools.styles.none')}</p>;
	const reset = async (frames: string[]) => {
		try {
			const done = await client.bookToolFix(id, { tool: 'resetStyles', frames });
			say(t('tools.styles.reset', { count: done.changed }));
		} catch {
			say(t('tools.styles.notReset'));
		}
	};
	return (
		<div>
			<p className={styles.summary}>{t('tools.styles.summary', { count: report.frames.length })}</p>
			<ul className={styles.rules}>
				{report.frames.map((stray) => (
					<li key={stray.frame} className={styles.stray}>
						<span>
							<span className={styles.ruleName}>
								{labels[book.spreads.findIndex((s) => s.id === stray.spread)] ?? ''}
							</span>
							<span className={styles.hint}>
								{t('tools.styles.row', {
									style: stray.style,
									changes: stray.changes
										.map((c) => t(CHANGE[c] ?? 'tools.styles.change.size'))
										.join(', '),
								})}
							</span>
						</span>
						<button
							type="button"
							className={styles.button}
							onClick={() => void reset([stray.frame])}
						>
							{t('tools.styles.resetOne')}
						</button>
					</li>
				))}
			</ul>
			<button
				type="button"
				className={styles.primary}
				onClick={() => void reset(report.frames.map((f) => f.frame))}
			>
				{t('tools.styles.resetAll', { count: report.frames.length })}
			</button>
		</div>
	);
}
