// The wizard's third step: the title, the author and an ISBN if there is one
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useId } from 'react';
import { t } from '../i18n/messages';
import { isValidIsbn13 } from './isbn';
import styles from './steps.module.css';
import type { WizardDraft } from './wizardDraft';

interface TitleStepProps {
	draft: WizardDraft;
	onChange: (patch: Partial<WizardDraft>) => void;
}

export function TitleStep({ draft, onChange }: TitleStepProps) {
	const id = useId();
	const isbnBad = draft.isbn.trim() !== '' && !isValidIsbn13(draft.isbn);
	return (
		<div className={styles.questions}>
			<label className={styles.question}>
				<span className={styles.label}>{t('wizard.title.title')}</span>
				<input
					className={styles.input}
					value={draft.title}
					placeholder={t('wizard.title.placeholder')}
					onChange={(event) => onChange({ title: event.target.value })}
				/>
			</label>
			<label className={styles.question}>
				<span className={styles.label}>{t('wizard.title.subtitle')}</span>
				<input
					className={styles.input}
					value={draft.subtitle}
					onChange={(event) => onChange({ subtitle: event.target.value })}
				/>
			</label>
			<label className={styles.question}>
				<span className={styles.label}>{t('wizard.title.author')}</span>
				<input
					className={styles.input}
					value={draft.author}
					onChange={(event) => onChange({ author: event.target.value })}
				/>
			</label>
			<div className={styles.question}>
				<label className={styles.label} htmlFor={`${id}-isbn`}>
					{t('wizard.title.isbn')}
				</label>
				<input
					id={`${id}-isbn`}
					className={styles.input}
					value={draft.isbn}
					inputMode="numeric"
					aria-invalid={isbnBad || undefined}
					aria-describedby={`${id}-isbn-note`}
					onChange={(event) => onChange({ isbn: event.target.value })}
				/>
				<p
					id={`${id}-isbn-note`}
					className={isbnBad ? styles.problem : styles.hint}
					role={isbnBad ? 'alert' : undefined}
				>
					{isbnBad ? t('wizard.title.isbnBad') : t('wizard.title.isbnHint')}
				</p>
			</div>
		</div>
	);
}
