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

const CUSTOM = 'custom';

const round = (value: number) => Math.round(value * 1000) / 1000;

interface SizeStepProps {
	kind: KindInfo | undefined;
	draft: WizardDraft;
	onChange: (patch: Partial<WizardDraft>) => void;
}

export function SizeStep({ kind, draft, onChange }: SizeStepProps) {
	if (!kind) return <p className={styles.note}>{t('wizard.size.none')}</p>;
	const custom = draft.customTrim;
	const spine = kind.spines.find((s) => s.paper === draft.paper)?.inches;
	return (
		<div className={styles.questions}>
			<div className={styles.question}>
				<span className={styles.label}>{t('wizard.size.trim')}</span>
				<Segmented
					label={t('wizard.size.trim')}
					value={custom ? CUSTOM : String(draft.trimIndex)}
					options={[
						...kind.trims.map((choice, index) => ({
							value: String(index),
							label: choice.label,
						})),
						{ value: CUSTOM, label: t('wizard.size.custom') },
					]}
					onChange={(value) => {
						if (value !== CUSTOM) {
							onChange({ trimIndex: Number(value), customTrim: undefined });
							return;
						}
						// Start from the size that was chosen, in inches.
						const from = kind.trims[draft.trimIndex] ?? kind.trims[0];
						onChange({
							customTrim: from
								? { width: round(from.width / 72), height: round(from.height / 72) }
								: { width: 6, height: 9 },
						});
					}}
				/>
				{custom && (
					<div className={styles.pair}>
						{(['width', 'height'] as const).map((side) => (
							<label key={side} className={styles.inline}>
								<span className={styles.label}>{t(`wizard.size.${side}`)}</span>
								<input
									className={styles.input}
									type="number"
									step="0.125"
									min="0"
									value={custom[side]}
									onChange={(event) =>
										onChange({ customTrim: { ...custom, [side]: Number(event.target.value) } })
									}
								/>
							</label>
						))}
					</div>
				)}
				{custom && <p className={styles.hint}>{t('wizard.size.customHint')}</p>}
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
				{spine !== undefined && (
					<p className={styles.hint} role="status">
						{t('wizard.size.spine', { pages: kind.defaultPages, inches: spine.toFixed(2) })}
					</p>
				)}
			</div>
		</div>
	);
}
