// The kinds of file the Export dialog offers, in the order they are shown
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ExportTarget } from '@liminal-hq/folio-protocol/generated/ExportTarget';
import type { MessageKey } from '../i18n/messages';

export interface TargetInfo {
	id: ExportTarget;
	name: MessageKey;
	description: MessageKey;
}

export const EXPORT_TARGETS: readonly TargetInfo[] = [
	{
		id: 'sharingPdf',
		name: 'export.target.sharingPdf',
		description: 'export.target.sharingPdf.sub',
	},
	{
		id: 'kindleEpub',
		name: 'export.target.kindleEpub',
		description: 'export.target.kindleEpub.sub',
	},
	{
		id: 'koboEpub',
		name: 'export.target.koboEpub',
		description: 'export.target.koboEpub.sub',
	},
	{
		id: 'kdpInterior',
		name: 'export.target.kdpInterior',
		description: 'export.target.kdpInterior.sub',
	},
];
