// Upload checklist: what a store will still ask for, with a copy button for what the book has
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ChecklistItem } from '@liminal-hq/folio-protocol/generated/ChecklistItem';
import { t } from '../../i18n/messages';
import type { ToolProps } from '../tools';
import { useToolReport } from '../useToolReport';
import styles from './publishing.module.css';

/** The items whose words are pasted into a store's form. */
const COPIED = new Set(['title', 'author', 'description', 'keywords', 'isbn']);

const MARK = { done: '✓', todo: '○', warning: '!' } as const;

export function ChecklistTool({ client, id, revision, say }: ToolProps) {
	const report = useToolReport(client, id, { tool: 'checklist' }, revision);
	if (report === null) return <p className={styles.note}>{t('tools.working')}</p>;
	const todo = report.items.filter((item) => item.status !== 'done').length;
	const copy = async (item: ChecklistItem) => {
		try {
			await navigator.clipboard.writeText(item.value ?? '');
			say(t('tools.checklist.copied', { what: t(`tools.checklist.item.${item.id}`) }));
		} catch {
			say(t('tools.checklist.notCopied'));
		}
	};
	return (
		<div>
			<p className={styles.summary}>
				{todo === 0 ? t('tools.checklist.ready') : t('tools.checklist.todo', { count: todo })}
			</p>
			<ul className={styles.list}>
				{report.items.map((item) => (
					<li key={item.id} className={styles.row} data-status={item.status}>
						<span className={styles.mark} aria-hidden="true">
							{MARK[item.status]}
						</span>
						<span className={styles.what}>
							<span className={styles.label}>{t(`tools.checklist.item.${item.id}`)}</span>
							<span className={styles.hint}>
								{t(
									item.status === 'done'
										? `tools.checklist.done.${item.id}`
										: item.id === 'isbn' && !item.value
											? 'tools.checklist.fix.isbnNone'
											: `tools.checklist.fix.${item.id}`,
									{ value: item.value ?? '' },
								)}
							</span>
							{item.value && COPIED.has(item.id) && (
								<span className={styles.value}>{item.value}</span>
							)}
						</span>
						{item.value && COPIED.has(item.id) && (
							<button type="button" className={styles.button} onClick={() => void copy(item)}>
								{t('tools.checklist.copy')}
							</button>
						)}
					</li>
				))}
			</ul>
		</div>
	);
}
