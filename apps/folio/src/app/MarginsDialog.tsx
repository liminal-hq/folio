// The margins dialog: the four margins of every page
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import { useEffect, useState } from 'react';
import { t } from '../i18n/messages';
import { LengthField } from '../inspector/LengthField';
import { unitLabel, type Units } from '../inspector/units';
import styles from './GeometryDialog.module.css';
import { marginsFit } from './geometryChecks';

interface MarginsDialogProps {
	open: boolean;
	onClose(): void;
	book: Book;
	units: Units;
	apply(intent: Intent): void;
}

const SIDES = ['top', 'bottom', 'inside', 'outside'] as const;

export function MarginsDialog({ open, onClose, book, units, apply }: MarginsDialogProps) {
	const [margins, setMargins] = useState(book.margins);
	useEffect(() => {
		if (open) setMargins(book.margins);
		// The form starts from the book each time the dialog opens.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [open]);

	const positive = SIDES.every((side) => margins[side] >= 0);
	const fits = positive && marginsFit(book.trim, margins);
	const changed = SIDES.some((side) => margins[side] !== book.margins[side]);
	const save = () => {
		if (changed) {
			apply({ type: 'SetBookGeometry', trim: null, margins, bleed: null, grid: null });
		}
		onClose();
	};

	return (
		<Dialog
			open={open}
			title={t('margins.title')}
			size="small"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton onClick={onClose}>{t('margins.cancel')}</DialogButton>
					<DialogButton variant="primary" disabled={!fits} onClick={save}>
						{t('margins.save')}
					</DialogButton>
				</DialogActions>
			}
		>
			<fieldset className={styles.fields}>
				<legend>
					{t('margins.legend')} ({unitLabel(units)})
				</legend>
				{SIDES.map((side) => (
					<LengthField
						key={side}
						label={t(`inspector.margin.${side}`)}
						points={margins[side]}
						units={units}
						onCommit={(points) => setMargins((now) => ({ ...now, [side]: points }))}
					/>
				))}
			</fieldset>
			{!positive && (
				<p className={styles.problem} role="alert">
					{t('margins.negative')}
				</p>
			)}
			{positive && !fits && (
				<p className={styles.problem} role="alert">
					{t('geometry.noRoom')}
				</p>
			)}
			<p className={styles.hint}>{t('margins.hint')}</p>
		</Dialog>
	);
}
