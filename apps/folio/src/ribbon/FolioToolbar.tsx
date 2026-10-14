// The toolbar under the title bar: the ribbon or the classic toolbar, whichever the person chose
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { ClassicToolbar } from '@liminal-hq/folio-ribbon/ClassicToolbar';
import { Ribbon } from '@liminal-hq/folio-ribbon/Ribbon';
import { useMemo } from 'react';
import type { RibbonCommand } from '@liminal-hq/folio-ribbon/types';
import type { CommandId } from '../commands/appMenuModel';
import { t } from '../i18n/messages';
import { useSettings, useUpdateSettings } from '../settings/useSettings';
import styles from './FolioToolbar.module.css';
import type { EditorState } from './editorCommands';
import { buildRibbonModel, buildToolbarModel } from './ribbonModel';
import { runRibbonCommand } from './runRibbonCommand';

export interface FolioToolbarProps {
	/** The menu commands that can be run now. */
	available: ReadonlySet<CommandId>;
	/** Runs a menu command by id. */
	run: (id: string) => Promise<void>;
	/** Ids of the contextual tabs to show; none until something on the page can be selected. */
	contextual?: readonly string[];
	/** The document's controls, when a book is open. */
	editor?: EditorState;
	/** Runs a document control; answers whether it took the command. */
	onEditorCommand?: (command: RibbonCommand) => boolean;
}

const NO_CONTEXTUAL: readonly string[] = [];

export function FolioToolbar({
	available,
	run,
	contextual = NO_CONTEXTUAL,
	editor,
	onEditorCommand,
}: FolioToolbarProps) {
	const settings = useSettings();
	const update = useUpdateSettings();
	const context = useMemo(() => ({ settings, available, editor }), [settings, available, editor]);
	const ribbon = useMemo(() => buildRibbonModel(context), [context]);
	const toolbar = useMemo(() => buildToolbarModel(context), [context]);
	const labels = useMemo(
		() => ({
			ribbon: t('region.toolbar'),
			toolbar: t('region.toolbar'),
			tabs: t('ribbon.tabs'),
			mode: t('mode.label'),
			modeEasy: t('mode.easy'),
			modeExpert: t('mode.expert'),
			pin: t('ribbon.pin'),
			unpin: t('ribbon.unpin'),
			decrease: t('ribbon.decrease'),
			increase: t('ribbon.increase'),
		}),
		[],
	);
	const onCommand = (command: Parameters<typeof runRibbonCommand>[0]) =>
		void runRibbonCommand(command, run, update, onEditorCommand);

	if (settings.interface.toolbar === 'classic') {
		return (
			<ClassicToolbar
				model={toolbar}
				mode={settings.mode}
				onCommand={onCommand}
				labels={labels}
				label={t('region.toolbar')}
				end={
					<button
						type="button"
						className={styles.export}
						disabled={!available.has('file.export')}
						title={available.has('file.export') ? undefined : t('ribbon.unavailable')}
						onClick={() => void run('file.export')}
					>
						{t('ribbon.item.exportBook')}
					</button>
				}
			/>
		);
	}
	return (
		<Ribbon
			model={ribbon}
			mode={settings.mode}
			contextual={contextual}
			onCommand={onCommand}
			labels={labels}
		/>
	);
}
