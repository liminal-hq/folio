// Asks whether to restore the unsaved work a crash left on a book, or open the book as it was saved
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { Recoverable } from '@liminal-hq/folio-plugin-books';
import { useCallback, useRef, useState, type ReactNode } from 'react';
import type { RecoveredChoice } from '../books/bookActions';
import { systemQuestions, type Questions } from '../dialogs/questions';
import { useSettings } from '../settings/useSettings';
import { t } from '../i18n/messages';
import { keptWhen } from '../welcome/recovered';

/**
 * A question to put to the person, and the dialog that asks it. `ask` resolves with what they
 * chose; Esc, a click outside and Cancel all mean cancel.
 */
export function useRecoveryQuestion(questions: Questions = systemQuestions): {
	ask: (found: Recoverable) => Promise<RecoveredChoice>;
	dialog: ReactNode;
} {
	const [asking, setAsking] = useState<Recoverable | null>(null);
	const answer = useRef<((choice: RecoveredChoice) => void) | null>(null);
	// With dialogs in their own windows the question is the system's message dialog.
	const native = useSettings().dialogs.ownWindows;

	const ask = useCallback(
		(found: Recoverable) =>
			native
				? questions.ask<RecoveredChoice>({
						title: t('recovery.title', { title: found.title }),
						text: t('recovery.message', { when: keptWhen(found.savedAt) }),
						answers: [
							{ value: 'restore', label: t('recovery.restoreIt') },
							{ value: 'file', label: t('recovery.file') },
							{ value: 'cancel', label: t('recovery.cancel') },
						],
					})
				: new Promise<RecoveredChoice>((resolve) => {
						answer.current = resolve;
						setAsking(found);
					}),
		[native, questions],
	);

	const settle = (choice: RecoveredChoice) => {
		answer.current?.(choice);
		answer.current = null;
		setAsking(null);
	};

	const dialog = (
		<Dialog
			open={!native && asking !== null}
			title={t('recovery.title', { title: asking?.title ?? '' })}
			description={t('recovery.message', { when: asking ? keptWhen(asking.savedAt) : '' })}
			size="small"
			onClose={() => settle('cancel')}
			footer={
				<DialogActions>
					<DialogButton onClick={() => settle('cancel')}>{t('recovery.cancel')}</DialogButton>
					<DialogButton variant="danger" onClick={() => settle('file')}>
						{t('recovery.file')}
					</DialogButton>
					<DialogButton variant="primary" onClick={() => settle('restore')}>
						{t('recovery.restoreIt')}
					</DialogButton>
				</DialogActions>
			}
		/>
	);

	return { ask, dialog };
}
