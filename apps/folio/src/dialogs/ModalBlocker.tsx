// Covers a window while a dialog window has it blocked, so a platform that does not block it still is
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useState } from 'react';
import { LIVE_PREVIEW_LABELS } from './dialogKinds';
import { tauriDialogsClient, type DialogsClient } from './dialogsClient';
import styles from './ModalBlocker.module.css';

/**
 * Where the platform blocks the window itself the cover only dims it; where it does not (some
 * compositors treat a modal as an ordinary child), the cover takes the pointer so the window cannot
 * be used, and a click brings the dialog forward.
 */
export function ModalBlocker({ client = tauriDialogsClient }: { client?: DialogsClient }) {
	const [blocked, setBlocked] = useState(false);
	// A dialog whose changes show on the page behind it leaves the page undimmed, so colours read true.
	const [clear, setClear] = useState(false);
	useEffect(() => {
		let live = true;
		let stop: (() => void) | undefined;
		client
			.onModal((state) => {
				if (!live) return;
				setBlocked(state.open);
				setClear(LIVE_PREVIEW_LABELS.includes(state.label));
			})
			.then((off) => (live ? (stop = off) : off()))
			.catch(() => {});
		return () => {
			live = false;
			stop?.();
		};
	}, [client]);
	if (!blocked) return null;
	return (
		<div
			className={clear ? `${styles.blocker} ${styles.clear}` : styles.blocker}
			data-testid="modal-blocker"
			onPointerDown={(event) => {
				event.preventDefault();
				void client.focus().catch(() => {});
			}}
			onContextMenu={(event) => event.preventDefault()}
			onWheel={(event) => event.preventDefault()}
			aria-hidden="true"
		/>
	);
}
