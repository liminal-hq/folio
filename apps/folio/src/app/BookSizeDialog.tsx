// The book size dialog: the width and height of the trimmed page
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import { useEffect, useState } from 'react';
import { DialogSurface } from '../dialogs/DialogSurface';
import { t } from '../i18n/messages';
import { LengthField } from '../inspector/LengthField';
import { unitLabel, type Units } from '../inspector/units';
import styles from './GeometryDialog.module.css';
import { marginsFit } from './geometryChecks';

interface BookSizeDialogProps {
	open: boolean;
	onClose(): void;
	book: Book;
	units: Units;
	apply(intent: Intent): void;
}

export function BookSizeDialog({ open, onClose, book, units, apply }: BookSizeDialogProps) {
	const [size, setSize] = useState({ w: book.trim.w, h: book.trim.h });
	useEffect(() => {
		if (open) setSize({ w: book.trim.w, h: book.trim.h });
		// The form starts from the book each time the dialog opens.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [open]);

	const sized = size.w > 0 && size.h > 0;
	const fits = sized && marginsFit(size, book.margins);
	const changed = size.w !== book.trim.w || size.h !== book.trim.h;
	const save = () => {
		if (changed) {
			apply({
				type: 'SetBookGeometry',
				trim: size,
				margins: null,
				bleed: null,
				grid: null,
			});
		}
		onClose();
	};

	return (
		<DialogSurface
			open={open}
			title={t('bookSize.title')}
			size="small"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton onClick={onClose}>{t('bookSize.cancel')}</DialogButton>
					<DialogButton variant="primary" disabled={!fits} onClick={save}>
						{t('bookSize.save')}
					</DialogButton>
				</DialogActions>
			}
		>
			<fieldset className={styles.fields}>
				<legend>
					{t('bookSize.trim')} ({unitLabel(units)})
				</legend>
				<LengthField
					label={t('bookSize.width')}
					points={size.w}
					units={units}
					onCommit={(w) => setSize((now) => ({ ...now, w }))}
				/>
				<LengthField
					label={t('bookSize.height')}
					points={size.h}
					units={units}
					onCommit={(h) => setSize((now) => ({ ...now, h }))}
				/>
			</fieldset>
			{!sized && (
				<p className={styles.problem} role="alert">
					{t('bookSize.none')}
				</p>
			)}
			{sized && !fits && (
				<p className={styles.problem} role="alert">
					{t('geometry.noRoom')}
				</p>
			)}
			<p className={styles.hint}>{t('bookSize.hint')}</p>
		</DialogSurface>
	);
}
