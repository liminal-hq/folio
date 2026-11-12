// The Spread templates dialog as a window of its own: it hands back the template that was chosen
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { tauriBooksClient } from '../books/booksClient';
import type { BooksClient } from '../books/booksClient';
import type { DialogPayloads } from '../dialogs/dialogKinds';
import type { DialogSession } from '../dialogs/DialogHost';
import { SpreadTemplatesDialog } from './SpreadTemplatesDialog';

export function SpreadTemplatesDialogWindow({
	payload,
	session,
	client = tauriBooksClient,
}: {
	payload: DialogPayloads['SpreadTemplates'];
	session: DialogSession;
	client?: BooksClient;
}) {
	return (
		<SpreadTemplatesDialog
			open
			onClose={session.close}
			client={client}
			id={payload.bookId}
			// Where the template goes depends on the spread the main window shows, so the main window
			// adds it and the window only says which one.
			onChoose={async (template, name) => session.finish({ template, name })}
		/>
	);
}
