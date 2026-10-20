// The wizard's fifth step: Easy or Expert, and how much help from Jax
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Segmented } from '../app/Segmented';
import { t } from '../i18n/messages';
import styles from './steps.module.css';
import type { WizardDraft } from './wizardDraft';

interface WayStepProps {
	draft: WizardDraft;
	onChange: (patch: Partial<WizardDraft>) => void;
}

export function WayStep({ draft, onChange }: WayStepProps) {
	return (
		<div className={styles.questions}>
			<div className={styles.question}>
				<span className={styles.label}>{t('mode.label')}</span>
				<Segmented
					label={t('mode.label')}
					value={draft.mode}
					options={[
						{ value: 'easy', label: t('mode.easy') },
						{ value: 'expert', label: t('mode.expert') },
					]}
					onChange={(mode) => onChange({ mode })}
				/>
				<p className={styles.hint}>{t('wizard.way.modeHint')}</p>
			</div>
			<div className={styles.question}>
				<span className={styles.label}>{t('welcome.jax')}</span>
				<Segmented
					label={t('welcome.jax')}
					value={draft.jax}
					options={[
						{ value: 'guide', label: t('preferences.jax.guide') },
						{ value: 'tips', label: t('preferences.jax.tips') },
						{ value: 'off', label: t('preferences.jax.off') },
					]}
					onChange={(jax) => onChange({ jax })}
				/>
			</div>
		</div>
	);
}
