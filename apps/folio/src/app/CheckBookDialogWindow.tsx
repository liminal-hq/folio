// The Check book list as a window beside the book: "Show me" moves the main window's canvas
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { shelfLabels } from '../books/shelfLabels';
import type { DialogMessages } from '../dialogs/dialogKinds';
import type { DialogSession } from '../dialogs/DialogHost';
import { useDialogBook } from '../dialogs/useDialogBook';
import { CheckBookDialog } from './CheckBookDialog';

export function CheckBookDialogWindow({ session }: { session: DialogSession }) {
	const dialog = useDialogBook(session);
	if (!dialog) return null;
	return (
		<CheckBookDialog
			open
			onClose={dialog.close}
			client={dialog.client}
			id={dialog.id}
			book={dialog.book}
			revision={dialog.revision}
			labels={shelfLabels(dialog.book)}
			onIgnore={(keys) => dialog.apply({ type: 'SetPreflightIgnore', keys })}
			// The book is in the main window, which moves to the issue; this window stays.
			keepOpenOnShow
			onShow={(spread, frame) => {
				const message: DialogMessages['CheckBook'] = { type: 'show', spread, frame };
				void session.client.post(message).catch(() => {});
			}}
		/>
	);
}
