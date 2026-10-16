// The languages a book can be written in, from the dictionaries Folio knows
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DictionaryStatus } from '@liminal-hq/folio-protocol/generated/DictionaryStatus';
import { t } from '../i18n/messages';

export interface LanguageChoice {
	/** The language code the book is given. */
	value: string;
	label: string;
}

const same = (a: string, b: string) => a.toLowerCase() === b.toLowerCase();

/**
 * One choice for each dictionary, named as it is and saying so when it has not been downloaded. The
 * language a dictionary gives a book is the first it checks. The book's own language comes along
 * when no dictionary claims it, so the control never shows a language the book is not in.
 */
export function languageChoices(
	dictionaries: DictionaryStatus[],
	current: string,
): { choices: LanguageChoice[]; selected: string } {
	const choices = dictionaries
		.filter((d) => d.languages.length > 0)
		.map((d) => ({
			value: d.languages[0]!,
			label: d.installed ? d.name : t('language.notDownloaded', { name: d.name }),
			languages: d.languages,
		}));
	// An exact code wins ("en-CA" is the bundled English); otherwise a dictionary that lists the
	// plain language ("fr" for a book in "fr-CA"), then one for the language's own code.
	const own = (languages: string[]) => languages.some((l) => same(l, current));
	const primary = current.split('-')[0]!;
	const match =
		choices.find((c) => c.languages.some((l) => same(l, current))) ??
		choices.find((c) => c.languages.some((l) => same(l, primary)));
	const list: LanguageChoice[] = choices.map(({ value, label }) => ({ value, label }));
	if (!match && !choices.some((c) => own(c.languages)))
		list.push({ value: current, label: current });
	return { choices: list, selected: match?.value ?? current };
}
