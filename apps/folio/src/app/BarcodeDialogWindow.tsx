// The Barcode dialog as a window of its own: the same dialog, working on the book it was opened for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DialogSession } from '../dialogs/DialogHost';
import { useDialogBook } from '../dialogs/useDialogBook';
import { BarcodeDialog } from './BarcodeDialog';

export function BarcodeDialogWindow({ session }: { session: DialogSession }) {
	const dialog = useDialogBook(session);
	if (!dialog) return null;
	return <BarcodeDialog open onClose={dialog.close} book={dialog.book} apply={dialog.apply} />;
}
