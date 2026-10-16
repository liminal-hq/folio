// A list of the languages a book can be written in, which also picks its spelling dictionary
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useId } from 'react';
import { t } from '../i18n/messages';
import styles from './Inspector.module.css';
import type { LanguageChoice } from './languageChoices';

export interface LanguageSelectProps {
	label: string;
	choices: LanguageChoice[];
	value: string;
	onChange(language: string): void;
}

export function LanguageSelect({ label, choices, value, onChange }: LanguageSelectProps) {
	const id = useId();
	return (
		<div className={styles.field}>
			<label htmlFor={id}>{label}</label>
			<select
				id={id}
				className={styles.select}
				value={value}
				onChange={(event) => onChange(event.target.value)}
			>
				{choices.map((choice) => (
					<option key={choice.value} value={choice.value}>
						{choice.label}
					</option>
				))}
			</select>
			<span className={styles.note}>{t('language.hint')}</span>
		</div>
	);
}
