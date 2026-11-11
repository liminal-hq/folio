// The screen of a dialog window: reads what it was opened for and shows the dialog of its kind
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useState } from 'react';
import { ExportDialogWindow } from '../app/ExportDialogWindow';
import { DialogWindowProvider } from './DialogSurface';
import type { DialogPayloads, OpenableKind } from './dialogKinds';
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
	return <DialogWindowProvider>{body(kind, payload, session)}</DialogWindowProvider>;
}

function body(kind: OpenableKind, payload: unknown, session: DialogSession) {
	switch (kind) {
		case 'Export':
			return <ExportDialogWindow payload={payload as DialogPayloads['Export']} session={session} />;
	}
}
