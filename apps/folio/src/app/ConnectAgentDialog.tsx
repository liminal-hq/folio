// The Connect an agent dialog: whether agents can reach the open books, and adding Folio to an agent program
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { AgentStatus } from '@liminal-hq/folio-protocol/generated/AgentStatus';
import { useEffect, useState } from 'react';
import { t } from '../i18n/messages';
import type { AgentClient } from '../services/agentClient';
import styles from './ConnectAgentDialog.module.css';

interface ConnectAgentDialogProps {
	open: boolean;
	onClose(): void;
	client: AgentClient;
}

/** What happened when Folio was added to one program. */
interface Outcome {
	ok: boolean;
	message: string;
}

export function ConnectAgentDialog({ open, onClose, client }: ConnectAgentDialogProps) {
	const [status, setStatus] = useState<AgentStatus | null>(null);
	const [failed, setFailed] = useState(false);
	const [outcomes, setOutcomes] = useState<Record<string, Outcome>>({});
	useEffect(() => {
		if (!open) return;
		let live = true;
		setStatus(null);
		setFailed(false);
		setOutcomes({});
		client
			.status()
			.then((found) => live && setStatus(found))
			.catch(() => live && setFailed(true));
		return () => {
			live = false;
		};
	}, [open, client]);

	const add = (program: string) => {
		client
			.install(program)
			.then((message) => setOutcomes((o) => ({ ...o, [program]: { ok: true, message } })))
			.catch((reason: unknown) =>
				setOutcomes((o) => ({
					...o,
					[program]: { ok: false, message: String(reason) },
				})),
			);
	};

	return (
		<Dialog
			open={open}
			title={t('agents.title')}
			size="medium"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton variant="primary" onClick={onClose}>
						{t('agents.done')}
					</DialogButton>
				</DialogActions>
			}
		>
			<p className={styles.intro}>{t('agents.intro')}</p>
			{failed && (
				<p role="alert" className={styles.note}>
					{t('agents.failed')}
				</p>
			)}
			{status === null && !failed && (
				<p role="status" className={styles.note}>
					{t('agents.checking')}
				</p>
			)}
			{status !== null && (
				<>
					<p role="status" className={status.listening ? styles.ok : styles.note}>
						{status.listening ? t('agents.listening') : t('agents.notListening')}
					</p>
					{status.binary === null ? (
						<p className={styles.note}>{t('agents.noProgram')}</p>
					) : (
						<>
							<h3 className={styles.heading}>{t('agents.addTo')}</h3>
							<ul className={styles.list}>
								{status.programs.map((program) => {
									const outcome = outcomes[program.id];
									return (
										<li key={program.id} className={styles.row}>
											<span className={styles.name}>{program.label}</span>
											<DialogButton
												onClick={() => add(program.id)}
												aria-label={t('agents.addOne', { name: program.label })}
											>
												{t('agents.add')}
											</DialogButton>
											{outcome && (
												<p
													role={outcome.ok ? 'status' : 'alert'}
													className={outcome.ok ? styles.ok : styles.bad}
												>
													{outcome.message}
												</p>
											)}
										</li>
									);
								})}
							</ul>
						</>
					)}
				</>
			)}
		</Dialog>
	);
}
