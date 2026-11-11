// What each kind of dialog window is opened with and what it hands back
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DialogKind } from '@liminal-hq/folio-protocol/generated/DialogKind';
import type { ExportTarget } from '@liminal-hq/folio-protocol/generated/ExportTarget';
import type { ImportOutcome } from '../app/ImportReportDialog';

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
	SpreadTemplates: { bookId: string };
	ImportReport: { outcome: ImportOutcome };
	CoverSheet: { bookId: string };
	SoftProof: { bookId: string; spread: string };
	CheckBook: { bookId: string };
	PageBackground: {
		bookId: string;
		/** The spread whose pages are being set. */
		spread: string;
		/** The page asked about, or null for the whole spread. */
		page: number | null;
		/** The place to start on when it is not one of the pages. */
		scope: string | null;
	};
}

/** What a dialog window of each kind hands back when it ends with a result (not when it is closed). */
export interface DialogResults {
	/** The person asked to check the book first. */
	Export: { next: 'check' };
	BookDetails: never;
	Barcode: never;
	BookSize: never;
	Margins: never;
	/** The template the person chose, to be added after the spread on show. */
	SpreadTemplates: { template: string; name: string };
	/** Bring the file in again with this choice of Folio style for each of its styles. */
	ImportReport: { mapping: Record<string, string> };
	CoverSheet: never;
	SoftProof: never;
	CheckBook: never;
	PageBackground: never;
}

/** What a dialog window that stays open posts to the window that opened it. */
export interface DialogMessages {
	/** "Show me": go to this spread and select this frame. */
	CheckBook: { type: 'show'; spread: number; frame: string | null };
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
	SpreadTemplates: 'dialog-spread-templates',
	ImportReport: 'dialog-import-report',
	CoverSheet: 'dialog-cover-sheet',
	SoftProof: 'dialog-soft-proof',
	CheckBook: 'dialog-check-book',
	PageBackground: 'dialog-page-background',
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

/** Dialogs whose changes show on the book behind them, so the opener is not dimmed while they are open. */
export const LIVE_PREVIEW_LABELS: readonly string[] = ['dialog-page-background'];

/** The kinds whose window watches and changes the book its payload names (through a pinned books store). */
const BOOK_DIALOGS: readonly OpenableKind[] = [
	'BookDetails',
	'Barcode',
	'BookSize',
	'Margins',
	'CoverSheet',
	'CheckBook',
	'PageBackground',
];

export function isBookDialog(kind: OpenableKind): boolean {
	return BOOK_DIALOGS.includes(kind);
}
