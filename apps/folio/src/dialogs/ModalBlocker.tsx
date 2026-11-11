// Covers a window while a dialog window has it blocked, so a platform that does not block it still is
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useState } from 'react';
import { tauriDialogsClient, type DialogsClient } from './dialogsClient';
import styles from './ModalBlocker.module.css';

/**
 * Where the platform blocks the window itself the cover only dims it; where it does not (some
 * compositors treat a modal as an ordinary child), the cover takes the pointer so the window cannot
 * be used, and a click brings the dialog forward.
 */
export function ModalBlocker({ client = tauriDialogsClient }: { client?: DialogsClient }) {
	const [blocked, setBlocked] = useState(false);
	useEffect(() => {
		let live = true;
		let stop: (() => void) | undefined;
		client
			.onModal((state) => live && setBlocked(state.open))
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
			className={styles.blocker}
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
