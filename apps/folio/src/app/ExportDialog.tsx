// The Export dialog: which files to make, a look at what preflight says, and where they went
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { ExportProgress } from '@liminal-hq/folio-protocol/generated/ExportProgress';
import type { ExportReport } from '@liminal-hq/folio-protocol/generated/ExportReport';
import type { ExportTarget } from '@liminal-hq/folio-protocol/generated/ExportTarget';
import { useEffect, useRef, useState } from 'react';
import type { BooksClient } from '../books/booksClient';
import { t, type MessageKey } from '../i18n/messages';
import { DialogSurface } from '../dialogs/DialogSurface';
import styles from './ExportDialog.module.css';
import { EXPORT_TARGETS } from './exportTargets';
import { whereItGoes } from './uploadGuide';
import { ExportNotes } from '../export/ExportNotes';
import { useExportChoices } from '../export/useExportChoices';

const targetName = (id: ExportTarget) =>
	EXPORT_TARGETS.find((target) => target.id === id)?.name ?? 'export.title';

interface ExportDialogProps {
	open: boolean;
	onClose(): void;
	client: BooksClient;
	id: string;
	/** Targets to make the moment the dialog opens, for the Quick PDF button. */
	autoStart?: ExportTarget[] | null;
	/** Opens the Check book dialog, and closes this one. */
	onCheck(): void;
}

type Stage =
	| { kind: 'choose' }
	| { kind: 'working' }
	| { kind: 'done'; report: ExportReport }
	| { kind: 'failed'; message: string };

const size = (bytes: number) =>
	bytes >= 1024 * 1024
		? `${(bytes / 1024 / 1024).toFixed(1)} MB`
		: `${Math.max(1, Math.round(bytes / 1024))} KB`;

const fileName = (path: string) => path.split(/[\\/]/).pop() ?? path;

export function ExportDialog({
	open,
	onClose,
	client,
	id,
	autoStart = null,
	onCheck,
}: ExportDialogProps) {
	const [chosen, setChosen] = useState<ExportTarget[]>(['sharingPdf']);
	const [stage, setStage] = useState<Stage>({ kind: 'choose' });
	const [waiting, setWaiting] = useState<number | null>(null);
	// How many pictures have no description, which a tagged PDF would give a screen reader to read as "figure".
	const [undescribed, setUndescribed] = useState(0);
	const started = useRef(false);

	const [reached, setReached] = useState<ExportProgress | null>(null);
	const [cmyk, setCmyk] = useState(false);
	const [profile, setProfile] = useState('');
	const [showWhere, setShowWhere] = useState(false);
	// The export works from a copy of the book taken when it starts; the window stays open beside
	// the book, so the book can be edited meanwhile. Say so, as the files then show it as it was.
	const [edited, setEdited] = useState(false);
	const exporting = useRef(false);

	const colour =
		cmyk || profile.trim() ? { cmyk, profile: profile.trim() || undefined } : undefined;
	const exportChoices = useExportChoices(client, id, open, chosen, colour);

	const run = (targets: ExportTarget[]) => {
		if (targets.length === 0) return;
		setStage({ kind: 'working' });
		setReached(null);
		void exportChoices.remember(targets).catch(() => undefined);
		setEdited(false);
		exporting.current = true;
		client
			.exportBook(id, targets, colour, exportChoices.choices)
			.then((report) => setStage({ kind: 'done', report }))
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

	useEffect(() => {
		if (!open) return;
		let off: (() => void) | undefined;
		let live = true;
		void client
			.subscribe({
				onPatch: (event) => {
					if (event.id === id && exporting.current) setEdited(true);
				},
				onChanged: () => {},
			})
			.then((unlisten) => {
				if (live) off = unlisten;
				else unlisten();
			});
		return () => {
			live = false;
			off?.();
		};
	}, [open, client, id]);

	// Which file the export has reached, while it works.
	useEffect(() => {
		if (!open) return;
		let off: (() => void) | undefined;
		let live = true;
		void client
			.onExportProgress((progress) => live && setReached(progress))
			.then((unlisten) => {
				if (live) off = unlisten;
				else unlisten();
			});
		return () => {
			live = false;
			off?.();
		};
	}, [open, client]);

	useEffect(() => {
		if (!open) {
			started.current = false;
			setStage({ kind: 'choose' });
			return;
		}
		let live = true;
		setWaiting(null);
		setUndescribed(0);
		client
			.preflightBook(id)
			.then((items) => {
				if (!live) return;
				setWaiting(items.filter((i) => i.severity === 'error' && !i.ignored).length);
				setUndescribed(items.filter((i) => i.kind === 'altMissing' && !i.ignored).length);
			})
			.catch(() => undefined);
		if (autoStart && !started.current) {
			started.current = true;
			setChosen(autoStart);
			run(autoStart);
		}
		return () => {
			live = false;
		};
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [open]);

	const toggle = (target: ExportTarget) =>
		setChosen((now) => (now.includes(target) ? now.filter((x) => x !== target) : [...now, target]));

	return (
		<DialogSurface
			open={open}
			title={t('export.title')}
			size="medium"
			onClose={onClose}
			footer={
				<DialogActions>
					{stage.kind === 'done' ? (
						<DialogButton variant="primary" onClick={onClose}>
							{t('export.done')}
						</DialogButton>
					) : (
						<>
							<DialogButton onClick={onClose}>{t('export.cancel')}</DialogButton>
							<DialogButton
								variant="primary"
								disabled={stage.kind === 'working' || chosen.length === 0}
								onClick={() => run(chosen)}
							>
								{stage.kind === 'working' ? t('export.working') : t('export.make')}
							</DialogButton>
						</>
					)}
				</DialogActions>
			}
		>
			{stage.kind !== 'done' && waiting !== null && waiting > 0 && (
				<p role="status" className={styles.warning}>
					{waiting === 1 ? t('export.waiting.one') : t('export.waiting', { count: waiting })}{' '}
					<button type="button" className={styles.link} onClick={onCheck}>
						{t('export.checkFirst')}
					</button>
				</p>
			)}
			{stage.kind !== 'done' && exportChoices.tagged && undescribed > 0 && (
				<p role="status" className={styles.warning}>
					{undescribed === 1
						? t('export.undescribed.one')
						: t('export.undescribed', { count: undescribed })}{' '}
					<button type="button" className={styles.link} onClick={onCheck}>
						{t('export.undescribed.show')}
					</button>
				</p>
			)}
			{edited && (stage.kind === 'working' || stage.kind === 'done') && (
				<p role="status" className={styles.warning}>
					{t('export.edited')}
				</p>
			)}
			{stage.kind === 'working' && (
				<div className={styles.progress} role="status">
					<span>
						{reached
							? t('export.progress', {
									done: reached.done + 1,
									total: reached.total,
									name: t(targetName(reached.target)),
								})
							: t('export.working')}
					</span>
					<button type="button" className={styles.link} onClick={() => void client.cancelExport()}>
						{t('export.stop')}
					</button>
				</div>
			)}
			{stage.kind === 'failed' && (
				<p role="alert" className={styles.error}>
					{t('export.failed', { message: stage.message })}
				</p>
			)}
			{stage.kind === 'done' ? (
				<section aria-label={t('export.files')}>
					<p className={styles.ok}>
						{stage.report.cancelled
							? t('export.stopped', { count: stage.report.files.length })
							: t('export.made', { count: stage.report.files.length })}
					</p>
					<ul className={styles.files}>
						{stage.report.files.map((file) => (
							<li key={file.path} className={styles.file}>
								<span className={styles.fileText}>
									<span className={styles.fileName}>{fileName(file.path)}</span>
									<span className={styles.fileSize}>{size(file.bytes)}</span>
									{showWhere && whereItGoes(fileName(file.path)) && (
										<span className={styles.fileWhere}>
											{t(whereItGoes(fileName(file.path)) as MessageKey)}
										</span>
									)}
								</span>
								<button
									type="button"
									className={styles.show}
									onClick={() => void client.revealPath(file.path)}
								>
									{t('export.show')}
								</button>
							</li>
						))}
					</ul>
					<button
						type="button"
						className={styles.show}
						aria-expanded={showWhere}
						onClick={() => setShowWhere((on) => !on)}
					>
						{t(showWhere ? 'export.where.hide' : 'export.where.show')}
					</button>
					<p className={styles.where}>{stage.report.folder}</p>
					<ExportNotes notes={stage.report.notes} />
				</section>
			) : (
				<fieldset className={styles.targets} disabled={stage.kind === 'working'}>
					<legend className={styles.legend}>{t('export.which')}</legend>
					{EXPORT_TARGETS.map((target) => (
						<label key={target.id} className={styles.target}>
							<input
								type="checkbox"
								checked={chosen.includes(target.id)}
								onChange={() => toggle(target.id)}
							/>
							<span className={styles.targetText}>
								<span className={styles.targetName}>{t(target.name)}</span>
								<span className={styles.targetSub}>{t(target.description)}</span>
							</span>
						</label>
					))}
				</fieldset>
			)}
			{stage.kind !== 'done' && exportChoices.panel}
			{stage.kind !== 'done' && chosen.some((x) => x === 'kdpInterior' || x === 'kdpCover') && (
				<fieldset className={styles.colour} disabled={stage.kind === 'working'}>
					<legend className={styles.legend}>{t('export.colour')}</legend>
					<label className={styles.target}>
						<input type="checkbox" checked={cmyk} onChange={(e) => setCmyk(e.target.checked)} />
						<span className={styles.targetText}>
							<span className={styles.targetName}>{t('export.colour.cmyk')}</span>
							<span className={styles.targetSub}>{t('export.colour.cmyk.sub')}</span>
						</span>
					</label>
					<label className={styles.profile}>
						<span className={styles.targetName}>{t('export.colour.profile')}</span>
						<input
							type="text"
							value={profile}
							placeholder={t('export.colour.profile.hint')}
							onChange={(e) => setProfile(e.target.value)}
						/>
					</label>
				</fieldset>
			)}
		</DialogSurface>
	);
}
