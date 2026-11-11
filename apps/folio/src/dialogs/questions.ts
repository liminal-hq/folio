// Questions with two or three answers, asked with the system's own message dialog
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { message } from '@tauri-apps/plugin-dialog';

/** One answer to a question, and the words on its button. */
export interface Answer<T extends string> {
	value: T;
	label: string;
}

/** A question with its answers, the last of which is the one that means "never mind". */
export interface Question<T extends string> {
	title: string;
	text: string;
	/** In the order the buttons show: the main answer, the other, then cancel. */
	answers: [Answer<T>, Answer<T>, Answer<T>];
}

/** Puts a question to the person. It resolves with the answer chosen, or the last one when the dialog is dismissed. */
export interface Questions {
	ask<T extends string>(question: Question<T>): Promise<T>;
}

/** The system's message dialog: a real modal window of the platform, parented to the asking window. */
export const systemQuestions: Questions = {
	async ask<T extends string>(question: Question<T>) {
		const [yes, no, cancel] = question.answers;
		const chosen = await message(question.text, {
			title: question.title,
			kind: 'warning',
			buttons: { yes: yes.label, no: no.label, cancel: cancel.label },
		});
		return question.answers.find((a) => a.label === chosen)?.value ?? cancel.value;
	},
};
