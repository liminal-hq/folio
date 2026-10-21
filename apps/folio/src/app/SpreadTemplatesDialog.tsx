// The Spread templates dialog: pictures of ready-made spreads at this book's size, to add one
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { TemplateGallery } from '@liminal-hq/folio-protocol/generated/TemplateGallery';
import { useEffect, useState } from 'react';
import type { BooksClient } from '../books/booksClient';
import { t } from '../i18n/messages';
import styles from './SpreadTemplatesDialog.module.css';

interface SpreadTemplatesDialogProps {
	open: boolean;
	onClose(): void;
	client: BooksClient;
	id: string;
	/** Adds the chosen template; the dialog closes when it has gone in, and says why when it could not. */
	onChoose(template: string, name: string): Promise<void>;
}

const ALL = '';

export function SpreadTemplatesDialog({
	open,
	onClose,
	client,
	id,
	onChoose,
}: SpreadTemplatesDialogProps) {
	const [gallery, setGallery] = useState<TemplateGallery | null | undefined>(undefined);
	const [group, setGroup] = useState(ALL);
	const [failed, setFailed] = useState(false);
	const [problem, setProblem] = useState<string | null>(null);

	useEffect(() => {
		if (!open) return;
		let live = true;
		setGallery(undefined);
		setFailed(false);
		setProblem(null);
		client
			.templateGallery(id)
			.then((found) => {
				if (!live) return;
				setGallery(found);
				// The gallery opens on the group of the book's own kind.
				setGroup(found.home);
			})
			.catch(() => live && setFailed(true));
		return () => {
			live = false;
		};
	}, [open, client, id]);

	const groups = gallery ? [...new Set(gallery.templates.map((card) => card.group))] : [];
	const shown = gallery?.templates.filter((card) => group === ALL || card.group === group) ?? [];

	return (
		<Dialog
			open={open}
			title={t('templates.title')}
			description={t('templates.hint')}
			size="large"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton onClick={onClose}>{t('templates.cancel')}</DialogButton>
				</DialogActions>
			}
		>
			{failed && (
				<p role="alert" className={styles.problem}>
					{t('templates.failed')}
				</p>
			)}
			{problem && (
				<p role="alert" className={styles.problem}>
					{problem}
				</p>
			)}
			{gallery === undefined && !failed && (
				<p role="status" className={styles.waiting}>
					{t('templates.drawing')}
				</p>
			)}
			{gallery && (
				<>
					<div className={styles.chips} role="group" aria-label={t('templates.groups')}>
						{[ALL, ...groups].map((name) => (
							<button
								key={name || 'all'}
								type="button"
								className={styles.chip}
								aria-pressed={group === name}
								onClick={() => setGroup(name)}
							>
								{name === ALL ? t('templates.all') : name}
							</button>
						))}
					</div>
					<ul className={styles.grid}>
						{shown.map((card) => (
							<li key={card.id}>
								<button
									type="button"
									className={styles.card}
									onClick={() => {
										setProblem(null);
										onChoose(card.id, card.name).catch((error: unknown) =>
											setProblem(error instanceof Error ? error.message : t('templates.refused')),
										);
									}}
								>
									<img className={styles.picture} src={card.image} alt="" />
									<span className={styles.name}>{card.name}</span>
									<span className={styles.group}>{card.group}</span>
								</button>
							</li>
						))}
					</ul>
				</>
			)}
		</Dialog>
	);
}
