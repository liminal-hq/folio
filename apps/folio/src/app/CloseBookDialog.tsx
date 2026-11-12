// Asks what to do with a book's changes before it closes
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { BookSummary } from '@liminal-hq/folio-plugin-books';
import { useCallback, useRef, useState, type ReactNode } from 'react';
import type { CloseChoice } from '../books/bookActions';
import { systemQuestions, type Questions } from '../dialogs/questions';
import { useSettings } from '../settings/useSettings';
import { t } from '../i18n/messages';

/**
 * A question to put to the person, and the dialog that asks it. `ask` resolves with what they
 * chose; Esc, a click outside and Cancel all mean cancel.
 */
export function useCloseQuestion(questions: Questions = systemQuestions): {
	ask: (summary: BookSummary) => Promise<CloseChoice>;
	dialog: ReactNode;
} {
	const [asking, setAsking] = useState<BookSummary | null>(null);
	const answer = useRef<((choice: CloseChoice) => void) | null>(null);
	// With dialogs in their own windows the question is the system's message dialog.
	const native = useSettings().dialogs.ownWindows;

	const ask = useCallback(
		(summary: BookSummary) =>
			native
				? questions.ask<CloseChoice>({
						title: t('close.title', { title: summary.title }),
						text: t('close.message'),
						answers: [
							{ value: 'save', label: t('close.save') },
							{ value: 'discard', label: t('close.discard') },
							{ value: 'cancel', label: t('close.cancel') },
						],
					})
				: new Promise<CloseChoice>((resolve) => {
						answer.current = resolve;
						setAsking(summary);
					}),
		[native, questions],
	);

	const settle = (choice: CloseChoice) => {
		answer.current?.(choice);
		answer.current = null;
		setAsking(null);
	};

	const dialog = (
		<Dialog
			open={!native && asking !== null}
			title={t('close.title', { title: asking?.title ?? '' })}
			description={t('close.message')}
			size="small"
			onClose={() => settle('cancel')}
			footer={
				<DialogActions>
					<DialogButton onClick={() => settle('cancel')}>{t('close.cancel')}</DialogButton>
					<DialogButton variant="danger" onClick={() => settle('discard')}>
						{t('close.discard')}
					</DialogButton>
					<DialogButton variant="primary" onClick={() => settle('save')}>
						{t('close.save')}
					</DialogButton>
				</DialogActions>
			}
		/>
	);

	return { ask, dialog };
}
