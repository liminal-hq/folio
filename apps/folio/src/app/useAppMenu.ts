// The app menu's rows and what choosing one does, for the settings and the books
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MenuItem, SelectableMenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import { useWindowControls } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import type { BookSummary } from '@liminal-hq/folio-plugin-books';
import { useCallback, useMemo } from 'react';
import type { CloseChoice } from '../books/bookActions';
import { useActiveBook, useBooks } from '../books/useBooks';
import { buildAppMenu, buildMenuBar } from '../commands/appMenuModel';
import {
	availableCommands,
	runCommand,
	type BookCommands,
	type CommandContext,
} from '../commands/runCommand';
import { useSettings, useUpdateSettings } from '../settings/useSettings';
import { useWindows } from './WindowsContext';

export interface AppMenuModel {
	/** The app menu's rows. */
	items: MenuItem[];
	/** The menu bar's menus. */
	barItems: MenuItem[];
	onSelect: (item: SelectableMenuItem) => void;
	/** Runs a command by id, as a key does. */
	run: (id: string) => Promise<void>;
}

/** `ask` puts the "save changes?" question to the person when a book with changes would close. */
export function useAppMenu(ask: (summary: BookSummary) => Promise<CloseChoice>): AppMenuModel {
	const settings = useSettings();
	const update = useUpdateSettings();
	const controls = useWindowControls();
	const windows = useWindows();
	const { store, actions } = useBooks();
	const active = useActiveBook();

	const available = useMemo(
		() =>
			availableCommands({
				hasBook: active !== null,
				canUndo: active?.copy.canUndo ?? false,
				canRedo: active?.copy.canRedo ?? false,
			}),
		[active],
	);
	const items = useMemo(
		() => buildAppMenu(settings, { enabled: available }),
		[settings, available],
	);
	const barItems = useMemo(
		() => buildMenuBar(settings, { enabled: available }),
		[settings, available],
	);

	const books = useMemo<BookCommands>(() => {
		/** Closes the book being shown, if there is one; `false` when the person said to keep it. */
		const makeRoom = async () => !active || (await actions.close(ask)) === 'done';
		return {
			newBook: async () => void (await makeRoom()),
			async open(path) {
				// Choose first, then close: cancelling the picker must not lose the book being shown.
				const target = path ?? (await actions.pick());
				if (target === null || !(await makeRoom())) return;
				await actions.open(target);
			},
			save: async () => void (await actions.save()),
			saveAs: async () => void (await actions.saveAs()),
			close: async () => void (await actions.close(ask)),
			undo: () => store.undo(),
			redo: () => store.redo(),
		};
	}, [active, actions, ask, store]);

	const run = useCallback(
		(id: string) => {
			const context: CommandContext = {
				settings,
				update,
				openWindow: (kind) => windows.open(kind),
				closeWindow: () => controls.close(),
				books,
			};
			return runCommand(id, context, available);
		},
		[settings, update, windows, controls, books, available],
	);

	const onSelect = useCallback((item: SelectableMenuItem) => void run(item.id), [run]);

	return { items, barItems, onSelect, run };
}
