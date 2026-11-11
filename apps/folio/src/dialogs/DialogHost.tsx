// The screen of a dialog window: reads what it was opened for and shows the dialog of its kind
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useState } from 'react';
import { BarcodeDialogWindow } from '../app/BarcodeDialogWindow';
import { BookDetailsDialogWindow } from '../app/BookDetailsDialogWindow';
import { BookSizeDialogWindow } from '../app/BookSizeDialogWindow';
import { CheckBookDialogWindow } from '../app/CheckBookDialogWindow';
import { PageBackgroundDialogWindow } from '../app/PageBackgroundDialogWindow';
import { StoryEditorDialogWindow } from '../app/StoryEditorDialogWindow';
import { CoverSheetDialogWindow } from '../app/CoverSheetDialogWindow';
import { ImportReportDialogWindow } from '../app/ImportReportDialogWindow';
import { SoftProofDialogWindow } from '../app/SoftProofDialogWindow';
import { SpreadTemplatesDialogWindow } from '../app/SpreadTemplatesDialogWindow';
import { ExportDialogWindow } from '../app/ExportDialogWindow';
import { MarginsDialogWindow } from '../app/MarginsDialogWindow';
import { BooksProvider } from '../books/BooksProvider';
import { tauriBooksClient } from '../books/booksClient';
import { tauriFileDialogs } from '../books/dialogs';
import { DialogWindowProvider } from './DialogSurface';
import { isBookDialog, type DialogPayloads, type OpenableKind } from './dialogKinds';
import { tauriDialogsClient, type DialogsClient } from './dialogsClient';

/**
 * What a dialog's body gets to work with. */
export interface DialogSession {
	client: DialogsClient;
	/** Ends the dialog with a result the opener hears. */
	finish(result: unknown): void;
	/** Closes the dialog; the opener hears a cancellation. */
	close(): void;
}

export function DialogHost({
	kind,
	client = tauriDialogsClient,
}: {
	kind: OpenableKind;
	client?: DialogsClient;
}) {
	const [payload, setPayload] = useState<unknown>(undefined);

	// The dialog asks for its payload as it starts, so it cannot miss it; once it has painted it
	// says so, which is how the start-up time is measured.
	useEffect(() => {
		let live = true;
		client
			.request()
			.then((request) => live && setPayload(request.payload))
			.catch(() => {});
		return () => {
			live = false;
		};
	}, [client]);
	useEffect(() => {
		if (payload === undefined) return;
		const frame = requestAnimationFrame(() =>
			requestAnimationFrame(() => void client.shown().catch(() => {})),
		);
		return () => cancelAnimationFrame(frame);
	}, [client, payload]);

	if (payload === undefined) return null;
	const session: DialogSession = {
		client,
		finish: (result) => void client.finish(result).catch(() => {}),
		close: () => void client.close().catch(() => {}),
	};
	return (
		<DialogWindowProvider>
			{isBookDialog(kind) ? (
				// The book the dialog was opened for, whichever book the main window shows now.
				<BooksProvider
					client={tauriBooksClient}
					dialogs={tauriFileDialogs}
					pin={(payload as { bookId: string }).bookId}
				>
					{body(kind, payload, session)}
				</BooksProvider>
			) : (
				body(kind, payload, session)
			)}
		</DialogWindowProvider>
	);
}

function body(kind: OpenableKind, payload: unknown, session: DialogSession) {
	switch (kind) {
		case 'Export':
			return <ExportDialogWindow payload={payload as DialogPayloads['Export']} session={session} />;
		case 'BookDetails':
			return <BookDetailsDialogWindow session={session} />;
		case 'Barcode':
			return <BarcodeDialogWindow session={session} />;
		case 'BookSize':
			return <BookSizeDialogWindow session={session} />;
		case 'Margins':
			return <MarginsDialogWindow session={session} />;
		case 'CoverSheet':
			return <CoverSheetDialogWindow session={session} />;
		case 'SpreadTemplates':
			return (
				<SpreadTemplatesDialogWindow
					payload={payload as DialogPayloads['SpreadTemplates']}
					session={session}
				/>
			);
		case 'ImportReport':
			return (
				<ImportReportDialogWindow
					payload={payload as DialogPayloads['ImportReport']}
					session={session}
				/>
			);
		case 'CheckBook':
			return <CheckBookDialogWindow session={session} />;
		case 'PageBackground':
			return (
				<PageBackgroundDialogWindow
					payload={payload as DialogPayloads['PageBackground']}
					session={session}
				/>
			);
		case 'StoryEditor':
			return (
				<StoryEditorDialogWindow
					payload={payload as DialogPayloads['StoryEditor']}
					session={session}
				/>
			);
		case 'SoftProof':
			return (
				<SoftProofDialogWindow payload={payload as DialogPayloads['SoftProof']} session={session} />
			);
	}
}
