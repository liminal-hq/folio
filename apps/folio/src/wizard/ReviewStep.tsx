// The wizard's last step: what will be made, and the button that makes it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { KindInfo } from '@liminal-hq/folio-plugin-books';
import { t } from '../i18n/messages';
import styles from './steps.module.css';
import type { WizardDraft } from './wizardDraft';

interface ReviewStepProps {
	kind: KindInfo | undefined;
	draft: WizardDraft;
	problem: string | null;
}

export function ReviewStep({ kind, draft, problem }: ReviewStepProps) {
	const trim = kind?.trims[draft.trimIndex]?.label ?? '';
	const rows: Array<[string, string]> = [
		[t('wizard.review.kind'), kind?.label ?? ''],
		[t('wizard.review.size'), trim],
		[t('wizard.review.paper'), t(`wizard.paper.${draft.paper}`)],
		[
			t('wizard.review.title'),
			draft.title.trim() || kind?.sampleTitle || t('wizard.review.untitled'),
		],
		[t('wizard.review.author'), draft.author.trim() || t('wizard.review.none')],
		[t('wizard.review.isbn'), draft.isbn.trim() || t('wizard.review.none')],
		[
			t('wizard.review.words'),
			draft.words === 'samples' ? t('wizard.words.samples') : t('wizard.words.empty'),
		],
		[t('wizard.review.mode'), draft.mode === 'easy' ? t('mode.easy') : t('mode.expert')],
		[t('wizard.review.look'), t(`wizard.look.${draft.look}`)],
	];
	return (
		<div className={styles.questions}>
			<dl className={styles.summary}>
				{rows.map(([name, value]) => (
					<div key={name} className={styles.summaryRow}>
						<dt className={styles.label}>{name}</dt>
						<dd className={styles.summaryValue}>{value}</dd>
					</div>
				))}
				<div className={styles.summaryRow}>
					<dt className={styles.label}>{t('wizard.review.cover')}</dt>
					<dd className={styles.summaryValue}>
						<span className={styles.chip} style={{ background: draft.coverColour }} />
						{draft.coverColour}
					</dd>
				</div>
			</dl>
			{problem && (
				<p role="alert" className={styles.problem}>
					{problem}
				</p>
			)}
		</div>
	);
}
