// Runs a menu command: the commands with behaviour today change settings or close the window
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Settings, SettingsPatch } from '@liminal-hq/folio-plugin-settings';
import type { WindowKind } from '@liminal-hq/folio-protocol/generated/WindowKind';
import type { CommandId } from './appMenuModel';

/** What a command needs from its window. */
export interface CommandContext {
	settings: Settings;
	update(patch: SettingsPatch): Promise<void>;
	/** Opens a window, or focuses it when it is already open. */
	openWindow(kind: WindowKind): void | Promise<void>;
	/** Closes the window the command was run in. */
	closeWindow(): void | Promise<void>;
}

/** The commands that have behaviour today. The menus list the others as unavailable. */
export const ENABLED_COMMANDS: ReadonlySet<CommandId> = new Set<CommandId>([
	'view.menuBar',
	'view.ribbon',
	'view.expert',
	'view.jax',
	'app.bookTools',
	'app.preferences',
	'app.about',
	'app.quit',
]);

/** Whether `id` is a command this module knows how to run. */
export function isCommandId(id: string): id is CommandId {
	return (ENABLED_COMMANDS as ReadonlySet<string>).has(id);
}

/** Runs the command `id`; an unknown or unavailable id does nothing. */
export async function runCommand(id: string, context: CommandContext): Promise<void> {
	if (!isCommandId(id)) return;
	const { settings } = context;
	switch (id) {
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
		case 'app.bookTools':
			return context.openWindow('BookTools');
		case 'app.preferences':
			return context.openWindow('Preferences');
		case 'app.about':
			return context.openWindow('About');
		case 'app.quit':
			await context.closeWindow();
			return;
		default:
			return;
	}
}
