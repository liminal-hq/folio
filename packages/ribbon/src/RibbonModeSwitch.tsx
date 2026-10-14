// The Easy and Expert switch at the end of the tab row or the classic toolbar
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { RibbonLabels } from './labels';
import styles from './Ribbon.module.css';
import type { RibbonMode } from './types';

export function RibbonModeSwitch({
	mode,
	labels,
	onChange,
}: {
	mode: RibbonMode;
	labels: RibbonLabels;
	onChange: (mode: RibbonMode) => void;
}) {
	return (
		<div role="radiogroup" aria-label={labels.mode} className={styles.mode}>
			{(['easy', 'expert'] as const).map((m) => (
				<button
					key={m}
					type="button"
					role="radio"
					aria-checked={mode === m}
					className={styles.modeOption}
					onClick={() => mode !== m && onChange(m)}
				>
					{m === 'easy' ? labels.modeEasy : labels.modeExpert}
				</button>
			))}
		</div>
	);
}
