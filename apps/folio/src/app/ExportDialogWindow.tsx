// The Export dialog as a window of its own: the same dialog, opened with a book to export
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { tauriBooksClient } from '../books/booksClient';
import type { DialogPayloads } from '../dialogs/dialogKinds';
import type { DialogSession } from '../dialogs/DialogHost';
import { ExportDialog } from './ExportDialog';

export function ExportDialogWindow({
	payload,
	session,
}: {
	payload: DialogPayloads['Export'];
	session: DialogSession;
}) {
	return (
		<ExportDialog
			open
			onClose={session.close}
			client={tauriBooksClient}
			id={payload.bookId}
			autoStart={payload.autoStart}
			// The window is the result: it ends with the answer, and the main window opens Check book.
			onCheck={() => session.finish({ next: 'check' })}
		/>
	);
}
