// The BookDetails dialog as a window of its own: the same dialog, working on the book it was opened for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DialogSession } from '../dialogs/DialogHost';
import { useDialogBook } from '../dialogs/useDialogBook';
import { BookDetailsDialog } from './BookDetailsDialog';

export function BookDetailsDialogWindow({ session }: { session: DialogSession }) {
	const dialog = useDialogBook(session);
	if (!dialog) return null;
	return (
		<BookDetailsDialog
			open
			onClose={dialog.close}
			book={dialog.book}
			onSave={(details) => dialog.apply({ type: 'SetDetails', details })}
			onPaper={(paper) => dialog.apply({ type: 'SetPaper', paper })}
		/>
	);
}
