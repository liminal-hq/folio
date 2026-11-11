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
	BookDetails: { bookId: string };
	Barcode: { bookId: string };
	BookSize: { bookId: string };
	Margins: { bookId: string };
}

/** What a dialog window of each kind hands back when it ends with a result (not when it is closed). */
export interface DialogResults {
	/** The person asked to check the book first. */
	Export: { next: 'check' };
	BookDetails: never;
	Barcode: never;
	BookSize: never;
	Margins: never;
}

/** The dialog kinds this build can open as windows. */
export type OpenableKind = keyof DialogPayloads & DialogKind;

/** The webview label of the window for each kind; mirrors `DialogKind::label` in `crates/folio-protocol`. */
const LABELS: Record<OpenableKind, string> = {
	Export: 'dialog-export',
	BookDetails: 'dialog-book-details',
	Barcode: 'dialog-barcode',
	BookSize: 'dialog-book-size',
	Margins: 'dialog-margins',
};

/** The kind a webview label denotes, or null when it is not a dialog window. */
export function dialogKindFromLabel(label: string): OpenableKind | null {
	const found = (Object.keys(LABELS) as OpenableKind[]).find((kind) => LABELS[kind] === label);
	return found ?? null;
}

/** The webview label of the window for a kind. */
export function dialogLabel(kind: OpenableKind): string {
	return LABELS[kind];
}

/** The kinds whose payload names a book, which the dialog watches and changes. */
export function isBookDialog(kind: OpenableKind): boolean {
	return kind !== 'Export';
}
