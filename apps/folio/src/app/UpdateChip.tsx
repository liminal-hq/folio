// A small control at the right of the toolbar for when there is something to do about updates
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { t } from '../i18n/messages';
import { useSettings } from '../settings/useSettings';
import { tauriUpdateClient, type UpdateClient } from '../services/updateClient';
import styles from './UpdateChip.module.css';
import { requestUpdateDialog } from './updateRequest';
import { useUpdateState } from './useUpdateState';

/**
 * Says nothing while there is nothing to do. It speaks up for a newer version, a download under way,
 * an update ready to install, and once, to offer automatic checks to someone who has not been asked.
 */
export function UpdateChip({ client = tauriUpdateClient }: { client?: UpdateClient }) {
	const state = useUpdateState(client);
	const { updates } = useSettings();
	switch (state?.kind) {
		case 'available':
			return (
				<button type="button" className={styles.chip} onClick={() => requestUpdateDialog()}>
					{t('update.chip.available', { version: state.version })}
				</button>
			);
		case 'downloading': {
			const percent =
				state.total !== null && state.total > 0
					? Math.min(100, Math.floor((state.received / state.total) * 100))
					: null;
			return (
				<span
					className={styles.chip}
					role="progressbar"
					aria-label={t('update.chip.downloading')}
					aria-valuemin={0}
					aria-valuemax={100}
					aria-valuenow={percent ?? undefined}
				>
					{percent === null
						? t('update.chip.downloading')
						: t('update.chip.downloading.percent', { percent })}
				</span>
			);
		}
		case 'ready':
			return (
				<button type="button" className={styles.chip} onClick={() => requestUpdateDialog()}>
					{t('update.chip.ready')}
				</button>
			);
		case 'idle':
			return updates.asked ? null : (
				<button type="button" className={styles.chip} onClick={() => requestUpdateDialog()}>
					{t('update.chip.offer')}
				</button>
			);
		default:
			return null;
	}
}
