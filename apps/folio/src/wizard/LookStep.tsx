// The wizard's sixth step: the typeface the book starts in, and the cover's colour
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Look } from '@liminal-hq/folio-protocol/generated/Look';
import '../canvas/fonts.css';
import { t } from '../i18n/messages';
import styles from './steps.module.css';
import type { WizardDraft } from './wizardDraft';

interface LookStepProps {
	draft: WizardDraft;
	onChange: (patch: Partial<WizardDraft>) => void;
}

/** The looks, each shown in its own typeface (the families `folio-presets` starts the styles in). */
const LOOKS: ReadonlyArray<{ value: Look; label: string; sub: string; family: string }> = [
	{
		value: 'friendly',
		label: t('wizard.look.friendly'),
		sub: t('wizard.look.friendlySub'),
		family: 'Atkinson Hyperlegible',
	},
	{
		value: 'modern',
		label: t('wizard.look.modern'),
		sub: t('wizard.look.modernSub'),
		family: 'Inter',
	},
	{
		value: 'classic',
		label: t('wizard.look.classic'),
		sub: t('wizard.look.classicSub'),
		family: 'Literata',
	},
];

/** Cover colours to start from; any colour can be chosen with the picker beside them. */
export const COVER_COLOURS: ReadonlyArray<{ value: string; name: string }> = [
	{ value: '#5b2a4a', name: 'Plum' },
	{ value: '#1f4e5f', name: 'Deep teal' },
	{ value: '#2f5d3a', name: 'Forest' },
	{ value: '#a23b2a', name: 'Brick' },
	{ value: '#c98b2b', name: 'Marigold' },
	{ value: '#2b3a67', name: 'Navy' },
	{ value: '#4a4a4a', name: 'Slate' },
	{ value: '#e9dcc3', name: 'Parchment' },
];

export function LookStep({ draft, onChange }: LookStepProps) {
	return (
		<div className={styles.questions}>
			<div className={styles.question}>
				<span className={styles.label}>{t('wizard.look.type')}</span>
				<div className={styles.choices} role="radiogroup" aria-label={t('wizard.look.type')}>
					{LOOKS.map((look) => (
						<button
							key={look.value}
							type="button"
							role="radio"
							aria-checked={draft.look === look.value}
							className={styles.choice}
							onClick={() => onChange({ look: look.value })}
						>
							<span className={styles.choiceTitle} style={{ fontFamily: look.family }}>
								{look.label}
							</span>
							<span className={styles.hint} style={{ fontFamily: look.family }}>
								{look.sub}
							</span>
						</button>
					))}
				</div>
			</div>
			<div className={styles.question}>
				<span className={styles.label}>{t('wizard.look.cover')}</span>
				<div className={styles.swatches} role="radiogroup" aria-label={t('wizard.look.cover')}>
					{COVER_COLOURS.map((colour) => (
						<button
							key={colour.value}
							type="button"
							role="radio"
							aria-checked={draft.coverColour === colour.value}
							aria-label={colour.name}
							title={colour.name}
							className={styles.swatch}
							style={{ background: colour.value }}
							onClick={() => onChange({ coverColour: colour.value })}
						/>
					))}
					<input
						type="color"
						className={styles.picker}
						aria-label={t('wizard.look.custom')}
						value={draft.coverColour}
						onChange={(event) => onChange({ coverColour: event.target.value })}
					/>
				</div>
			</div>
		</div>
	);
}
