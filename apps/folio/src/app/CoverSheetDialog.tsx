// The cover sheet preview: back, spine and front as the paperback cover file will have them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import { useEffect, useState } from 'react';
import type { BooksClient } from '../books/booksClient';
import { t } from '../i18n/messages';
import styles from './CoverSheetDialog.module.css';

interface CoverSheetDialogProps {
	open: boolean;
	onClose(): void;
	client: BooksClient;
	id: string;
	/** Changes when the book does, so the sheet is drawn again. */
	revision: number;
}

export function CoverSheetDialog({ open, onClose, client, id, revision }: CoverSheetDialogProps) {
	const [picture, setPicture] = useState<string | null | undefined>(undefined);
	const [failed, setFailed] = useState(false);
	useEffect(() => {
		if (!open) return;
		let live = true;
		setPicture(undefined);
		setFailed(false);
		client
			.previewCoverSheet(id)
			.then((found) => live && setPicture(found))
			.catch(() => live && setFailed(true));
		return () => {
			live = false;
		};
	}, [open, client, id, revision]);

	return (
		<Dialog
			open={open}
			title={t('coverSheet.title')}
			size="large"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton variant="primary" onClick={onClose}>
						{t('coverSheet.done')}
					</DialogButton>
				</DialogActions>
			}
		>
			<p className={styles.hint}>{t('coverSheet.hint')}</p>
			{failed && (
				<p role="alert" className={styles.problem}>
					{t('coverSheet.failed')}
				</p>
			)}
			{picture === undefined && !failed && (
				<p role="status" className={styles.hint}>
					{t('coverSheet.drawing')}
				</p>
			)}
			{picture === null && <p className={styles.hint}>{t('coverSheet.none')}</p>}
			{picture && <img className={styles.sheet} src={picture} alt={t('coverSheet.alt')} />}
		</Dialog>
	);
}
