// Runs a menu command: the settings toggles, the windows, the books, and Quit
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Settings, SettingsPatch } from '@liminal-hq/folio-plugin-settings';
import type { WindowKind } from '@liminal-hq/folio-protocol/generated/WindowKind';
import { requestAgentDialog } from '../app/agentRequest';
import { keepWords } from '../canvas/editCheckpoint';
import { RECENT_PREFIX, type CommandId } from './appMenuModel';

/** What the books can do for a command. Each answers when it is finished. */
export interface BookCommands {
	/** Opens a book from the picker, or from `path`, after asking about the book being shown. */
	open(path?: string): Promise<void>;
	save(): Promise<void>;
	saveAs(): Promise<void>;
	close(): Promise<void>;
	undo(): Promise<void>;
	redo(): Promise<void>;
	/** Shows the Export dialog on the book being shown. */
	exportBook(): Promise<void>;
	/** Asks for a Markdown or text file and brings its words into the book being shown. */
	importText(): Promise<void>;
}

/** What a command needs from its window. */
export interface CommandContext {
	settings: Settings;
	update(patch: SettingsPatch): Promise<void>;
	/** Opens a window, or focuses it when it is already open. */
	openWindow(kind: WindowKind): void | Promise<void>;
	/** Opens the Book tools window on a group of tools, or moves it there when it is open. */
	openBookTools(area: string): void | Promise<void>;
	/** Closes the window the command was run in. */
	closeWindow(): void | Promise<void>;
	books: BookCommands;
}

/** The commands that work whatever is open. */
export const ALWAYS_AVAILABLE: ReadonlySet<CommandId> = new Set<CommandId>([
	'file.new',
	'file.open',
	'view.menuBar',
	'view.ribbon',
	'view.expert',
	'view.jax',
	'tools.structure',
	'tools.typesetting',
	'tools.pictures',
	'tools.publishing',
	'app.preferences',
	'app.agents',
	'app.about',
	'app.quit',
]);

/** What the window has open, which decides which commands can be run. */
export interface CommandState {
	hasBook: boolean;
	canUndo: boolean;
	canRedo: boolean;
}

/** The commands that can be run now. The menus list the others as unavailable. */
export function availableCommands(state: CommandState): ReadonlySet<CommandId> {
	const available = new Set<CommandId>(ALWAYS_AVAILABLE);
	if (state.hasBook) {
		(['file.save', 'file.saveAs', 'file.close', 'file.export', 'file.importText'] as const).forEach(
			(id) => available.add(id),
		);
	}
	if (state.canUndo) available.add('edit.undo');
	if (state.canRedo) available.add('edit.redo');
	return available;
}

/** Runs the command `id` if it is one of `available`; anything else does nothing. */
export async function runCommand(
	id: string,
	context: CommandContext,
	available: ReadonlySet<string>,
): Promise<void> {
	if (id.startsWith(RECENT_PREFIX)) {
		return context.books.open(id.slice(RECENT_PREFIX.length));
	}
	if (!available.has(id)) return;
	const { settings, books } = context;
	switch (id as CommandId) {
		case 'view.menuBar':
			return context.update({ interface: { menuBar: !settings.interface.menuBar } });
		case 'view.ribbon':
			return context.update({
				interface: { toolbar: settings.interface.toolbar === 'ribbon' ? 'classic' : 'ribbon' },
			});
		case 'view.expert':
			return context.update({ mode: settings.mode === 'expert' ? 'easy' : 'expert' });
		case 'view.jax':
			// Switching Jax back on returns to the gentle default rather than the last choice.
			return context.update({ jax: settings.jax === 'off' ? 'tips' : 'off' });
		case 'tools.structure':
		case 'tools.typesetting':
		case 'tools.pictures':
		case 'tools.publishing':
			return context.openBookTools(id.slice('tools.'.length));
		case 'app.preferences':
			return context.openWindow('Preferences');
		case 'app.agents':
			return requestAgentDialog();
		case 'app.about':
			return context.openWindow('About');
		case 'app.quit':
			await context.closeWindow();
			return;
		case 'file.new':
			// The wizard makes the book and adds it beside any already open, so nothing is closed.
			return context.openWindow('WizardNew');
		case 'file.open':
			return books.open();
		case 'file.save':
			return books.save();
		case 'file.saveAs':
			return books.saveAs();
		case 'file.close':
			return books.close();
		case 'file.export':
			return books.exportBook();
		case 'file.importText':
			return books.importText();
		// Words being typed are kept first, and the editing closes, so the undo or redo that follows
		// acts on the book as it stands and not on the box.
		case 'edit.undo':
			keepWords(true);
			return books.undo();
		case 'edit.redo':
			keepWords(true);
			return books.redo();
		default:
			return;
	}
}
