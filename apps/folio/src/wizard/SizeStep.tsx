// The wizard's second step: the trim size and the paper
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { KindInfo } from '@liminal-hq/folio-plugin-books';
import type { Paper } from '@liminal-hq/folio-protocol/generated/Paper';
import { Segmented } from '../app/Segmented';
import { t } from '../i18n/messages';
import styles from './steps.module.css';
import type { WizardDraft } from './wizardDraft';

interface SizeStepProps {
	kind: KindInfo | undefined;
	draft: WizardDraft;
	onChange: (patch: Partial<WizardDraft>) => void;
}

export function SizeStep({ kind, draft, onChange }: SizeStepProps) {
	if (!kind) return <p className={styles.note}>{t('wizard.size.none')}</p>;
	return (
		<div className={styles.questions}>
			<div className={styles.question}>
				<span className={styles.label}>{t('wizard.size.trim')}</span>
				<Segmented
					label={t('wizard.size.trim')}
					value={String(draft.trimIndex)}
					options={kind.trims.map((choice, index) => ({
						value: String(index),
						label: choice.label,
					}))}
					onChange={(value) => onChange({ trimIndex: Number(value) })}
				/>
			</div>
			<div className={styles.question}>
				<span className={styles.label}>{t('wizard.size.paper')}</span>
				<Segmented<Paper>
					label={t('wizard.size.paper')}
					value={draft.paper}
					options={[
						{ value: 'white', label: t('wizard.paper.white') },
						{ value: 'cream', label: t('wizard.paper.cream') },
						{ value: 'colour', label: t('wizard.paper.colour') },
					]}
					onChange={(paper) => onChange({ paper })}
				/>
				<p className={styles.hint}>{t('wizard.size.paperHint')}</p>
			</div>
		</div>
	);
}
