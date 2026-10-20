// The Soft proof dialog: a spread as a press would print it, beside how it is drawn
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { ProofPage } from '@liminal-hq/folio-protocol/generated/ProofPage';
import { useEffect, useState } from 'react';
import type { BooksClient } from '../books/booksClient';
import { t } from '../i18n/messages';
import styles from './SoftProofDialog.module.css';

interface SoftProofDialogProps {
	open: boolean;
	onClose(): void;
	client: BooksClient;
	id: string;
	/** The spread being looked at. */
	spread: string;
}

type Stage =
	| { kind: 'choose' }
	| { kind: 'working' }
	| { kind: 'done'; pages: ProofPage[] }
	| { kind: 'failed'; message: string };

export function SoftProofDialog({ open, onClose, client, id, spread }: SoftProofDialogProps) {
	const [profile, setProfile] = useState('');
	const [stage, setStage] = useState<Stage>({ kind: 'choose' });
	const [asDrawn, setAsDrawn] = useState(false);

	useEffect(() => {
		if (!open) setStage({ kind: 'choose' });
	}, [open]);

	const show = () => {
		setStage({ kind: 'working' });
		client
			.proofSpread(id, spread, profile.trim())
			.then((pages) => setStage({ kind: 'done', pages }))
			.catch((reason: unknown) =>
				setStage({
					kind: 'failed',
					message:
						typeof reason === 'object' && reason !== null && 'message' in reason
							? String((reason as { message: unknown }).message)
							: String(reason),
				}),
			);
	};

	return (
		<Dialog
			open={open}
			title={t('proof.title')}
			size="medium"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton onClick={onClose}>{t('proof.close')}</DialogButton>
					<DialogButton
						variant="primary"
						disabled={stage.kind === 'working' || profile.trim() === ''}
						onClick={show}
					>
						{stage.kind === 'working' ? t('proof.working') : t('proof.show')}
					</DialogButton>
				</DialogActions>
			}
		>
			<p className={styles.intro}>{t('proof.intro')}</p>
			<label className={styles.field}>
				<span>{t('proof.profile')}</span>
				<input
					type="text"
					value={profile}
					placeholder="/path/to/press.icc"
					onChange={(event) => setProfile(event.target.value)}
				/>
			</label>
			{stage.kind === 'failed' && (
				<p role="alert" className={styles.error}>
					{t('proof.failed', { message: stage.message })}
				</p>
			)}
			{stage.kind === 'done' && (
				<section aria-label={t('proof.title')}>
					<label className={styles.toggle}>
						<input
							type="checkbox"
							checked={asDrawn}
							onChange={(event) => setAsDrawn(event.target.checked)}
						/>
						{t('proof.original')}
					</label>
					<div className={styles.pages}>
						{stage.pages.map((page, index) => (
							<figure key={index} className={styles.page}>
								<img
									src={asDrawn ? page.original : page.proofed}
									alt={`${t('proof.page', { page: index + 1 })} — ${asDrawn ? t('proof.original') : t('proof.proofed')}`}
								/>
								<figcaption>
									{page.outside < 0.05
										? t('proof.inside')
										: t('proof.outside', { percent: page.outside.toFixed(1) })}
								</figcaption>
							</figure>
						))}
					</div>
				</section>
			)}
		</Dialog>
	);
}
