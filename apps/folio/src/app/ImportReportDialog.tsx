// What a text import brought in, and what it left out; or why it could not
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { ImportReport } from '@liminal-hq/folio-protocol/generated/ImportReport';
import { useState } from 'react';
import { t, type MessageKey } from '../i18n/messages';
import styles from './ImportReportDialog.module.css';

/** The result of an import, or the reason it was refused. */
export type ImportOutcome = { report: ImportReport } | { problem: string };

interface ImportReportDialogProps {
	outcome: ImportOutcome | null;
	onClose(): void;
	/** Brings the same file in again with the person's choice of Folio style for each of its styles. */
	onRemap?(mapping: Record<string, string>): void;
}

/** The Folio styles a document's style can be given: its id, and the words for it. */
const CHOICES: [string, MessageKey][] = [
	['', 'import.style.body'],
	['h1', 'import.style.h1'],
	['h2', 'import.style.h2'],
	['epigraph', 'import.style.epigraph'],
];

export function ImportReportDialog({ outcome, onClose, onRemap }: ImportReportDialogProps) {
	/** What the person has changed since the import, by the document's style name. */
	const [changes, setChanges] = useState<Record<string, string>>({});
	const close = () => {
		setChanges({});
		onClose();
	};
	return (
		<Dialog
			open={outcome !== null}
			title={t('import.title')}
			size="medium"
			onClose={close}
			footer={
				<DialogActions>
					{onRemap && outcome && 'report' in outcome && Object.keys(changes).length > 0 && (
						<DialogButton
							onClick={() => {
								const mapping = Object.fromEntries(
									outcome.report.styles.map((s) => [s.name, changes[s.name] ?? s.folio ?? '']),
								);
								setChanges({});
								onRemap(mapping);
							}}
						>
							{t('import.style.apply')}
						</DialogButton>
					)}
					<DialogButton variant="primary" onClick={close}>
						{t('import.done')}
					</DialogButton>
				</DialogActions>
			}
		>
			{outcome && 'problem' in outcome && (
				<p role="alert" className={styles.problem}>
					{outcome.problem}
				</p>
			)}
			{outcome && 'report' in outcome && (
				<section aria-label={t('import.title')}>
					<p className={styles.ok}>
						{t('import.brought', {
							paragraphs: outcome.report.paragraphs,
							headings: outcome.report.headings,
							breaks: outcome.report.sceneBreaks,
						})}
					</p>
					{outcome.report.notes.length > 0 && (
						<>
							<p className={styles.lead}>{t('import.notes')}</p>
							<ul className={styles.notes}>
								{outcome.report.notes.map((note) => (
									<li key={note}>{note}</li>
								))}
							</ul>
						</>
					)}
					{onRemap && outcome.report.styles.length > 0 && (
						<>
							<p className={styles.lead}>{t('import.styles')}</p>
							<table className={styles.styles}>
								<tbody>
									{outcome.report.styles.map((style) => (
										<tr key={style.name}>
											<th scope="row">
												{style.name}
												<span className={styles.sample}>
													{t('import.style.count', { count: style.count })} · {style.sample}
												</span>
											</th>
											<td>
												<select
													aria-label={t('import.style.for', { name: style.name })}
													value={changes[style.name] ?? style.folio ?? ''}
													onChange={(event) =>
														setChanges({ ...changes, [style.name]: event.target.value })
													}
												>
													{CHOICES.map(([id, label]) => (
														<option key={id} value={id}>
															{t(label)}
														</option>
													))}
												</select>
											</td>
										</tr>
									))}
								</tbody>
							</table>
						</>
					)}
					<p className={styles.hint}>{t('import.undo')}</p>
				</section>
			)}
		</Dialog>
	);
}
