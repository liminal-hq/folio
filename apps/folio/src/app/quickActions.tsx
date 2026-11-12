// The title bar's quick actions: one list of commands, run through the same path as the app menu
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { QuickAction } from '@liminal-hq/folio-chrome/TitleBar/QuickActions';
import { useMemo, type ReactNode } from 'react';
import { FILE_SHORTCUTS, type CommandId } from '../commands/appMenuModel';
import { t, type MessageKey } from '../i18n/messages';
import { NewBookIcon, OpenIcon, SaveIcon } from '../icons/AppIcons';

/** One shortcut in the strip: a command the app menu also has, so there is a single behaviour. */
interface QuickActionSpec {
	/** The command it runs, by the id the app menu and the keys use. */
	command: CommandId;
	label: MessageKey;
	icon: ReactNode;
	shortcut: string;
	/** Whether the unsaved-changes dot shows on it while the book has changes. */
	showsUnsaved?: boolean;
}

/**
 * The quick actions, in order. This is the only place that says what the strip holds, so a later
 * setting can let people choose their own from the same commands. Undo and Redo are left out:
 * the ribbon has them.
 */
export const QUICK_ACTIONS: readonly QuickActionSpec[] = [
	{
		command: 'file.save',
		label: 'quickActions.save',
		icon: <SaveIcon />,
		shortcut: FILE_SHORTCUTS.save,
		showsUnsaved: true,
	},
	{
		command: 'file.new',
		label: 'quickActions.new',
		icon: <NewBookIcon />,
		shortcut: FILE_SHORTCUTS.new,
	},
	{
		command: 'file.open',
		label: 'quickActions.open',
		icon: <OpenIcon />,
		shortcut: FILE_SHORTCUTS.open,
	},
];

/**
 * The strip's buttons for the book being shown. `run` and `available` are the app menu's, so a
 * button does what its menu row does (Save on a book never saved asks where, as File → Save does).
 */
export function useQuickActions(
	run: (id: string) => Promise<void>,
	available: ReadonlySet<CommandId>,
	/** What is unsaved about the book shown: changes, never saved, or nothing. */
	unsaved: 'changes' | 'never' | null,
): QuickAction[] {
	return useMemo(
		() =>
			QUICK_ACTIONS.map((spec) => {
				const dot = spec.showsUnsaved === true && unsaved !== null && available.has(spec.command);
				return {
					id: spec.command,
					label: t(spec.label),
					icon: spec.icon,
					shortcut: spec.shortcut,
					disabled: !available.has(spec.command),
					...(dot
						? {
								badge: true,
								badgeLabel: t(
									unsaved === 'never' ? 'quickActions.save.never' : 'quickActions.save.unsaved',
								),
							}
						: {}),
					onSelect: () => void run(spec.command),
				};
			}),
		[run, available, unsaved],
	);
}
