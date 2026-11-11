// What each kind of dialog window is opened with and what it hands back
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DialogKind } from '@liminal-hq/folio-protocol/generated/DialogKind';
import type { ExportTarget } from '@liminal-hq/folio-protocol/generated/ExportTarget';

/** What the opener sends a dialog window of each kind to start from. */
export interface DialogPayloads {
	Export: {
		bookId: string;
		/** Targets to make the moment the dialog opens, for the Quick PDF button. */
		autoStart: ExportTarget[] | null;
	};
}

/** What a dialog window of each kind hands back when it ends with a result (not when it is closed). */
export interface DialogResults {
	/** The person asked to check the book first. */
	Export: { next: 'check' };
}

/** The dialog kinds this build can open as windows. */
export type OpenableKind = keyof DialogPayloads & DialogKind;

/** Mirrors `DialogKind::from_label` in `crates/folio-protocol`: the kind a webview label denotes. */
export function dialogKindFromLabel(label: string): OpenableKind | null {
	switch (label) {
		case 'dialog-export':
			return 'Export';
	}
	return null;
}

/** The webview label of the window for a kind. */
export function dialogLabel(kind: OpenableKind): string {
	switch (kind) {
		case 'Export':
			return 'dialog-export';
	}
}
