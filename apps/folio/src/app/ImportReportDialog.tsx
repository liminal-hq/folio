// What a text import brought in, and what it left out; or why it could not
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { ImportReport } from '@liminal-hq/folio-protocol/generated/ImportReport';
import { t } from '../i18n/messages';
import styles from './ImportReportDialog.module.css';

/** The result of an import, or the reason it was refused. */
export type ImportOutcome = { report: ImportReport } | { problem: string };

interface ImportReportDialogProps {
	outcome: ImportOutcome | null;
	onClose(): void;
}

export function ImportReportDialog({ outcome, onClose }: ImportReportDialogProps) {
	return (
		<Dialog
			open={outcome !== null}
			title={t('import.title')}
			size="medium"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton variant="primary" onClick={onClose}>
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
					<p className={styles.hint}>{t('import.undo')}</p>
				</section>
			)}
		</Dialog>
	);
}
