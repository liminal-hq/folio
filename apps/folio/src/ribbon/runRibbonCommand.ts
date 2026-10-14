// Turns what the ribbon reports into the menu commands and settings changes they stand for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { RibbonCommand } from '@liminal-hq/folio-ribbon/types';
import type { SettingsPatch } from '@liminal-hq/folio-plugin-settings';
import { RIBBON_COMMANDS } from './ribbonModel';

/** Runs a ribbon command: the mode switch, a File menu choice, or a control wired to a menu command. */
export async function runRibbonCommand(
	command: RibbonCommand,
	run: (id: string) => Promise<void>,
	update: (patch: SettingsPatch) => Promise<void>,
): Promise<void> {
	switch (command.id) {
		case 'mode':
			if (command.value === 'easy' || command.value === 'expert') {
				return update({ mode: command.value });
			}
			return;
		case 'tab':
			return;
		case 'file':
			if (typeof command.value === 'string') return run(command.value);
			return;
		default: {
			const id = RIBBON_COMMANDS[command.id];
			if (id) return run(id);
		}
	}
}
