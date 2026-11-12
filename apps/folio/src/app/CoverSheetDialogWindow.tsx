// The Cover sheet as a window of its own, working on the book it was opened for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DialogSession } from '../dialogs/DialogHost';
import { useDialogBook } from '../dialogs/useDialogBook';
import { CoverSheetDialog } from './CoverSheetDialog';
import { putDescriptionIntent } from './coverSheetIntents';

export function CoverSheetDialogWindow({ session }: { session: DialogSession }) {
	const dialog = useDialogBook(session);
	if (!dialog) return null;
	return (
		<CoverSheetDialog
			open
			onClose={dialog.close}
			client={dialog.client}
			id={dialog.id}
			revision={dialog.revision}
			book={dialog.book}
			onSave={(details) => dialog.apply({ type: 'SetDetails', details })}
			onPutDescription={(text) => {
				const intent = putDescriptionIntent(dialog.book, text);
				if (intent) dialog.apply(intent);
			}}
		/>
	);
}
