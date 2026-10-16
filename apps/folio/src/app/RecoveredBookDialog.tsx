// Asks whether to restore the unsaved work a crash left on a book, or open the book as it was saved
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { Recoverable } from '@liminal-hq/folio-plugin-books';
import { useCallback, useRef, useState, type ReactNode } from 'react';
import type { RecoveredChoice } from '../books/bookActions';
import { t } from '../i18n/messages';
import { keptWhen } from '../welcome/recovered';

/**
 * A question to put to the person, and the dialog that asks it. `ask` resolves with what they
 * chose; Esc, a click outside and Cancel all mean cancel.
 */
export function useRecoveryQuestion(): {
	ask: (found: Recoverable) => Promise<RecoveredChoice>;
	dialog: ReactNode;
} {
	const [asking, setAsking] = useState<Recoverable | null>(null);
	const answer = useRef<((choice: RecoveredChoice) => void) | null>(null);

	const ask = useCallback(
		(found: Recoverable) =>
			new Promise<RecoveredChoice>((resolve) => {
				answer.current = resolve;
				setAsking(found);
			}),
		[],
	);

	const settle = (choice: RecoveredChoice) => {
		answer.current?.(choice);
		answer.current = null;
		setAsking(null);
	};

	const dialog = (
		<Dialog
			open={asking !== null}
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
