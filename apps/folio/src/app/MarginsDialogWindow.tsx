// The Margins dialog as a window of its own: the same dialog, working on the book it was opened for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DialogSession } from '../dialogs/DialogHost';
import { useDialogBook } from '../dialogs/useDialogBook';
import { useSettings } from '../settings/useSettings';
import { MarginsDialog } from './MarginsDialog';

export function MarginsDialogWindow({ session }: { session: DialogSession }) {
	const dialog = useDialogBook(session);
	const { units } = useSettings();
	if (!dialog) return null;
	return (
		<MarginsDialog
			open
			onClose={dialog.close}
			book={dialog.book}
			units={units}
			apply={dialog.apply}
		/>
	);
}
