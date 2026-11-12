// The Import report as a window of its own: it hands back a new choice of styles when there is one
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DialogPayloads } from '../dialogs/dialogKinds';
import type { DialogSession } from '../dialogs/DialogHost';
import { ImportReportDialog } from './ImportReportDialog';

export function ImportReportDialogWindow({
	payload,
	session,
}: {
	payload: DialogPayloads['ImportReport'];
	session: DialogSession;
}) {
	return (
		<ImportReportDialog
			outcome={payload.outcome}
			onClose={session.close}
			onRemap={(mapping) => session.finish({ mapping })}
		/>
	);
}
