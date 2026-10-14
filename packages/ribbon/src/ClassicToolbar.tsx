// The classic toolbar: one row of icon buttons in groups, with the Easy and Expert switch at the end
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useMemo, type ReactNode } from 'react';
import { defaultRibbonLabels, type RibbonLabels } from './labels';
import styles from './Ribbon.module.css';
import { RibbonModeSwitch } from './RibbonModeSwitch';
import type { RibbonCommand, RibbonMode, ToolbarModel } from './types';
import { useRibbonControls } from './useRibbonControls';

export interface ClassicToolbarProps {
	model: ToolbarModel;
	mode: RibbonMode;
	onCommand: (command: RibbonCommand) => void;
	/** Controls placed after the mode switch, such as an Export button. */
	end?: ReactNode;
	/** Accessible name of the toolbar. */
	label?: string;
	labels?: Partial<RibbonLabels>;
}

export function ClassicToolbar({
	model,
	mode,
	onCommand,
	end,
	label,
	...props
}: ClassicToolbarProps) {
	const labels = { ...defaultRibbonLabels, ...props.labels };
	const peers = useMemo(() => model.groups.flat(), [model]);
	const controls = useRibbonControls(peers, labels, onCommand);
	return (
		<div role="toolbar" aria-label={label ?? labels.ribbon} className={styles.classic}>
			{model.groups.map((group, index) => (
				<div key={index} role="group" className={styles.classicGroup}>
					{group.map((item) => controls.control(item, false))}
				</div>
			))}
			<div className={styles.end}>
				<RibbonModeSwitch
					mode={mode}
					labels={labels}
					onChange={(next) => onCommand({ id: 'mode', value: next })}
				/>
				{end}
			</div>
			{controls.overlays}
		</div>
	);
}
