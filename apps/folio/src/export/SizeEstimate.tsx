// "About 3.8 MB": the size the chosen PDFs will be, worked out after a pause in the choosing
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ExportChoices } from '@liminal-hq/folio-plugin-books';
import type { ExportColour } from '@liminal-hq/folio-protocol/generated/ExportColour';
import type { ExportTarget } from '@liminal-hq/folio-protocol/generated/ExportTarget';
import { useEffect, useState } from 'react';
import type { BooksClient } from '../books/booksClient';
import { t } from '../i18n/messages';
import styles from './SizeEstimate.module.css';

/** How long the choosing must pause before the size is worked out again, in milliseconds. */
export const ESTIMATE_PAUSE_MS = 400;

const PDFS: ExportTarget[] = ['sharingPdf', 'kdpInterior', 'kdpCover', 'printShopPdf'];

/** A size as a person reads it. */
export function sizeText(bytes: number): string {
	return bytes >= 1024 * 1024
		? `${(bytes / 1024 / 1024).toFixed(1)} MB`
		: `${Math.max(1, Math.round(bytes / 1024))} KB`;
}

interface SizeEstimateProps {
	client: BooksClient;
	id: string;
	targets: ExportTarget[];
	colour: ExportColour | undefined;
	choices: ExportChoices;
	open: boolean;
}

export function SizeEstimate({ client, id, targets, colour, choices, open }: SizeEstimateProps) {
	const [bytes, setBytes] = useState<number | null>(null);
	const pdfs = targets.filter((target) => PDFS.includes(target));
	const key = JSON.stringify([id, pdfs, colour, choices]);

	useEffect(() => {
		if (!open || pdfs.length === 0) return;
		let live = true;
		setBytes(null);
		const timer = setTimeout(() => {
			client
				.estimateExport(id, pdfs, colour, choices)
				.then((estimate) => {
					// A newer ask stopped this one: its answer is not whole, and the newer one will come.
					if (live && !estimate.cancelled) setBytes(estimate.bytes);
				})
				.catch(() => undefined);
		}, ESTIMATE_PAUSE_MS);
		return () => {
			live = false;
			clearTimeout(timer);
		};
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [open, key, client]);

	if (pdfs.length === 0) return null;
	return (
		<p className={styles.estimate} role="status" aria-live="polite">
			{bytes === null
				? t('export.estimate.working')
				: t('export.estimate', { size: sizeText(bytes) })}
		</p>
	);
}
