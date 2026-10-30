// The Update dialog: what a newer version is, and fetching and installing it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import { useEffect, useState } from 'react';
import { t, type MessageKey } from '../i18n/messages';
import { useSettings, useUpdateSettings } from '../settings/useSettings';
import type { UpdateClient } from '../services/updateClient';
import styles from './UpdateDialog.module.css';
import { useUpdateState } from './useUpdateState';

interface UpdateDialogProps {
	open: boolean;
	onClose(): void;
	client: UpdateClient;
	/** Looks for a newer version as soon as the dialog opens. */
	checkOnOpen: boolean;
	/**
	 * Called before installing, which restarts Folio: asks about books with changes and answers
	 * whether to go on.
	 */
	beforeInstall(): Promise<boolean>;
}

export function UpdateDialog({
	open,
	onClose,
	client,
	checkOnOpen,
	beforeInstall,
}: UpdateDialogProps) {
	const state = useUpdateState(client) ?? { kind: 'idle' as const };
	const settings = useSettings();
	const update = useUpdateSettings();
	const [failed, setFailed] = useState<string | null>(null);

	useEffect(() => {
		if (!open) return;
		setFailed(null);
		if (checkOnOpen) void client.check().catch(() => undefined);
		// Opening the dialog answers the one-time offer, whichever way it is dismissed.
		if (!settings.updates.asked) void update({ updates: { asked: true } });
		// eslint-disable-next-line react-hooks/exhaustive-deps -- only when it opens
	}, [open]);

	const install = async () => {
		setFailed(null);
		if (!(await beforeInstall())) return;
		client.install().catch((reason: unknown) => setFailed(String(reason)));
	};

	const busy = state.kind === 'checking' || state.kind === 'downloading';
	return (
		<Dialog
			open={open}
			title={t('update.title')}
			size="medium"
			onClose={onClose}
			footer={
				<DialogActions>
					{state.kind === 'available' && (
						<DialogButton variant="primary" onClick={() => void client.download()}>
							{t('update.download')}
						</DialogButton>
					)}
					{state.kind === 'ready' && (
						<DialogButton variant="primary" onClick={() => void install()}>
							{t('update.install')}
						</DialogButton>
					)}
					{(state.kind === 'idle' || state.kind === 'upToDate' || state.kind === 'error') && (
						<DialogButton onClick={() => void client.check().catch(() => undefined)}>
							{t('update.check')}
						</DialogButton>
					)}
					<DialogButton
						variant={state.kind === 'available' || state.kind === 'ready' ? undefined : 'primary'}
						onClick={onClose}
					>
						{t('update.close')}
					</DialogButton>
				</DialogActions>
			}
		>
			{state.kind === 'idle' && <p className={styles.note}>{t('update.idle')}</p>}
			{state.kind === 'checking' && (
				<p role="status" className={styles.note}>
					{t('update.checking')}
				</p>
			)}
			{state.kind === 'upToDate' && (
				<p role="status" className={styles.ok}>
					{t('update.upToDate')}
				</p>
			)}
			{state.kind === 'unsupported' && (
				<p role="status" className={styles.note}>
					{state.reason}
				</p>
			)}
			{state.kind === 'error' && (
				<p role="alert" className={styles.bad}>
					{t('update.failed', { message: state.message })}
				</p>
			)}
			{state.kind === 'available' && (
				<>
					<p role="status" className={styles.ok}>
						{t('update.available', { version: state.version })}
					</p>
					{state.notes !== null && <pre className={styles.notes}>{state.notes}</pre>}
					<p className={styles.note}>{t(`update.how.${state.install}` as MessageKey)}</p>
				</>
			)}
			{state.kind === 'downloading' && (
				<p role="status" className={styles.note}>
					{t('update.downloading')}
				</p>
			)}
			{state.kind === 'ready' && (
				<p role="status" className={styles.ok}>
					{t('update.ready', { version: state.version })}
				</p>
			)}
			{failed !== null && (
				<p role="alert" className={styles.bad}>
					{t('update.failed', { message: failed })}
				</p>
			)}
			{!busy && state.kind !== 'unsupported' && (
				<label className={styles.automatic}>
					<input
						type="checkbox"
						checked={settings.updates.automatic}
						onChange={(e) => void update({ updates: { automatic: e.target.checked } })}
					/>
					<span>
						<span className={styles.automaticName}>{t('update.automatic')}</span>
						<span className={styles.note}>{t('update.automatic.sub')}</span>
					</span>
				</label>
			)}
		</Dialog>
	);
}
