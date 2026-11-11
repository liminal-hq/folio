// The BookSize dialog as a window of its own: the same dialog, working on the book it was opened for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DialogSession } from '../dialogs/DialogHost';
import { useDialogBook } from '../dialogs/useDialogBook';
import { useSettings } from '../settings/useSettings';
import { BookSizeDialog } from './BookSizeDialog';

export function BookSizeDialogWindow({ session }: { session: DialogSession }) {
	const dialog = useDialogBook(session);
	const { units } = useSettings();
	if (!dialog) return null;
	return (
		<BookSizeDialog
			open
			onClose={dialog.close}
			book={dialog.book}
			units={units}
			apply={dialog.apply}
		/>
	);
}
