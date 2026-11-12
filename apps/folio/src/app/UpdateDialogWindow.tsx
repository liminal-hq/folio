// The Update dialog as a window of its own: installing is the main window's to do, since it asks about books first
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DialogPayloads } from '../dialogs/dialogKinds';
import type { DialogSession } from '../dialogs/DialogHost';
import { tauriUpdateClient, type UpdateClient } from '../services/updateClient';
import { UpdateDialog } from './UpdateDialog';

export function UpdateDialogWindow({
	payload,
	session,
	client = tauriUpdateClient,
}: {
	payload: DialogPayloads['Update'];
	session: DialogSession;
	client?: UpdateClient;
}) {
	return (
		<UpdateDialog
			open
			onClose={session.close}
			client={client}
			checkOnOpen={payload.checkOnOpen}
			// Installing restarts Folio, so the books with changes are the main window's to ask about:
			// the window ends with "install" and the main window does the rest.
			beforeInstall={async () => {
				session.finish({ install: true });
				return false;
			}}
		/>
	);
}
