// The choices an export is made with: the person's defaults, the book's, and this export's own
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ExportChoices } from '@liminal-hq/folio-plugin-books';
import type { SettingsPatch } from '@liminal-hq/folio-plugin-settings';
import type { ExportColour } from '@liminal-hq/folio-protocol/generated/ExportColour';
import type { ExportPrefs } from '@liminal-hq/folio-protocol/generated/ExportPrefs';
import type { ExportTarget } from '@liminal-hq/folio-protocol/generated/ExportTarget';
import type { PdfPrefs } from '@liminal-hq/folio-protocol/generated/PdfPrefs';
import { useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import { createStore } from 'zustand/vanilla';
import { useStore } from 'zustand';
import type { BooksClient } from '../books/booksClient';
import { DEFAULT_SETTINGS } from '../settings/fakeSettingsClient';
import { SettingsContext } from '../settings/useSettings';
import { settingsOf, type SettingsState } from '../settings/settingsStore';
import { emptyPrefs, isBlank, kindOf, overlay, withPdf, type PdfKind } from './exportPrefs';
import { ExpertPdfOptions } from './ExpertPdfOptions';
import { PdfQuality } from './PdfQuality';
import { SizeEstimate } from './SizeEstimate';

/** Stands in for the settings in a window with none, so the hooks below always have a store. */
const NO_SETTINGS = createStore<SettingsState>(() => ({ snapshot: null, error: null }));

export interface ExportChoicesState {
	/** What the dialog shows for the pictures of each chosen kind of PDF, and the size it expects. */
	panel: ReactNode;
	/** To hand to `exportBook`: this export's choices and the person's defaults. */
	choices: ExportChoices;
	/** Keeps this export's choices in the book, as Export does when it starts. */
	remember(targets: ExportTarget[]): Promise<void>;
}

/**
 * Brings the choices of the Export dialog together. The dialog opens with the person's defaults
 * under the book's own choices; what is changed in it is this export's, saved to the book when
 * Export is pressed and to the person's defaults when they ask for that.
 */
export function useExportChoices(
	client: BooksClient,
	id: string,
	open: boolean,
	targets: ExportTarget[],
	colour: ExportColour | undefined,
): ExportChoicesState {
	const store = useContext(SettingsContext);
	const settings = useStore(store?.state ?? NO_SETTINGS, (state) =>
		settingsOf(state, DEFAULT_SETTINGS),
	);
	const user = settings.export;
	const [book, setBook] = useState<ExportPrefs>(emptyPrefs());
	const [edits, setEdits] = useState<ExportPrefs>(emptyPrefs());

	// The book's own choices are read each time the dialog opens, and what was changed is dropped.
	useEffect(() => {
		if (!open) return;
		let live = true;
		setEdits(emptyPrefs());
		client
			.snapshot(id)
			.then((snapshot) => live && setBook(snapshot.book.export ?? emptyPrefs()))
			.catch(() => undefined);
		return () => {
			live = false;
		};
	}, [open, client, id]);

	const change = useCallback(
		(kind: PdfKind, pdf: Partial<PdfPrefs>) => setEdits((now) => withPdf(now, kind, pdf)),
		[],
	);

	const remember = useCallback(
		async (made: ExportTarget[]) => {
			// The book's choices are read fresh: the dialog may have started before they were in.
			const current = (await client.snapshot(id)).book.export ?? emptyPrefs();
			const next = overlay(current, edits);
			next.lastTargets = made;
			setBook(next);
			setEdits(emptyPrefs());
			if (JSON.stringify(next) !== JSON.stringify(current)) {
				await client.apply(id, { type: 'SetExportPrefs', prefs: next });
			}
		},
		[edits, client, id],
	);

	const makeDefaults = useCallback(async () => {
		if (!store) return;
		await store.update({ export: { pdf: edits.pdf } } as SettingsPatch);
	}, [store, edits]);

	const choices = useMemo<ExportChoices>(() => ({ prefs: edits, user }), [edits, user]);
	const kinds = [...new Set(targets.map(kindOf).filter((k): k is PdfKind => k !== null))];
	const panel =
		kinds.length === 0 ? null : (
			<>
				<PdfQuality
					kinds={kinds}
					user={user}
					book={book}
					edits={edits}
					onChange={change}
					onMakeDefaults={store ? () => void makeDefaults() : undefined}
					changed={!isBlank(edits)}
				/>
				{settings.mode === 'expert' && (
					<ExpertPdfOptions kinds={kinds} user={user} book={book} edits={edits} onChange={change} />
				)}
				<SizeEstimate
					client={client}
					id={id}
					targets={targets}
					colour={colour}
					choices={choices}
					open={open}
				/>
			</>
		);
	return { panel, choices, remember };
}
