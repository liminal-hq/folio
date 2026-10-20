// The wizard's fourth step: start from sample pages, or from nothing
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Words } from '@liminal-hq/folio-protocol/generated/Words';
import { t } from '../i18n/messages';
import styles from './steps.module.css';
import type { WizardDraft } from './wizardDraft';

interface WordsStepProps {
	draft: WizardDraft;
	onChange: (patch: Partial<WizardDraft>) => void;
}

const CHOICES: ReadonlyArray<{ value: Words; label: string; sub: string }> = [
	{ value: 'samples', label: t('wizard.words.samples'), sub: t('wizard.words.samplesSub') },
	{ value: 'empty', label: t('wizard.words.empty'), sub: t('wizard.words.emptySub') },
];

export function WordsStep({ draft, onChange }: WordsStepProps) {
	return (
		<div className={styles.questions}>
			<div className={styles.choices} role="radiogroup" aria-label={t('wizard.step.words')}>
				{CHOICES.map((choice) => (
					<button
						key={choice.value}
						type="button"
						role="radio"
						aria-checked={draft.words === choice.value}
						className={styles.choice}
						onClick={() => onChange({ words: choice.value })}
					>
						<span className={styles.choiceTitle}>{choice.label}</span>
						<span className={styles.hint}>{choice.sub}</span>
					</button>
				))}
				<button type="button" role="radio" aria-checked="false" className={styles.choice} disabled>
					<span className={styles.choiceTitle}>{t('wizard.words.import')}</span>
					<span className={styles.hint}>{t('menu.unavailable')}</span>
				</button>
			</div>
		</div>
	);
}
