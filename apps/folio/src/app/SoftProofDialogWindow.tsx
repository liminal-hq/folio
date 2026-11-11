// The Soft proof as a window of its own: a look at one spread as the printer will see it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { tauriBooksClient } from '../books/booksClient';
import type { BooksClient } from '../books/booksClient';
import type { DialogPayloads } from '../dialogs/dialogKinds';
import type { DialogSession } from '../dialogs/DialogHost';
import { SoftProofDialog } from './SoftProofDialog';

export function SoftProofDialogWindow({
	payload,
	session,
	client = tauriBooksClient,
}: {
	payload: DialogPayloads['SoftProof'];
	session: DialogSession;
	client?: BooksClient;
}) {
	return (
		<SoftProofDialog
			open
			onClose={session.close}
			client={client}
			id={payload.bookId}
			spread={payload.spread}
		/>
	);
}
