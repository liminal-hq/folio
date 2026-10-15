// The dialog that sets a page's background: where it goes, then what it is
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { Background } from '@liminal-hq/folio-protocol/generated/Background';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { PaintPresets } from '@liminal-hq/folio-protocol/generated/PaintPresets';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { useEffect, useState } from 'react';
import { Segmented } from '../app/Segmented';
import { t } from '../i18n/messages';
import { ColourPicker } from '../inspector/ColourPicker';
import { backgroundTargets, currentBackground, type BackgroundScope } from './backgroundScopes';
import styles from './PageBackgroundDialog.module.css';

export interface PageBackgroundDialogProps {
	open: boolean;
	onClose(): void;
	book: Book;
	spread: Spread;
	/** The page the person asked about, so its scope is the one chosen first; `null` for the whole spread. */
	page: number | null;
	presets: PaintPresets | null;
	onApply(intent: Intent): void;
}

type Kind = 'colour' | 'gradient';

const kindOf = (bg: Background | null): Kind => (bg?.type === 'gradient' ? 'gradient' : 'colour');

const css = (bg: Extract<Background, { type: 'gradient' }>) =>
	`linear-gradient(${bg.angle}deg, ${bg.stops.map((s) => `${s.colour} ${s.at * 100}%`).join(', ')})`;

/**
 * Choosing a colour or a gradient applies it at once, so the page behind the dialog shows it; the
 * session joins the tries into one undo step. Done only closes.
 */
export function PageBackgroundDialog({
	open,
	onClose,
	book,
	spread,
	page,
	presets,
	onApply,
}: PageBackgroundDialogProps) {
	const scopes = backgroundTargets(spread);
	const first: BackgroundScope = scopes[page === null || scopes.length < 3 ? 0 : page + 1]!;
	const [chosen, setChosen] = useState<BackgroundScope>(first);
	const scope = scopes.find((s) => s.id === chosen.id) ?? scopes[0]!;
	const now = currentBackground(book, spread, scope);
	const [kind, setKind] = useState<Kind>(kindOf(now));

	// Opening it again starts from the page asked about, with the tab of what that page has.
	useEffect(() => {
		if (!open) return;
		setChosen(first);
		setKind(kindOf(currentBackground(book, spread, first)));
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [open, page, spread.id]);

	const set = (bg: Background | null) =>
		onApply({ type: 'SetBackground', target: scope.target, bg });

	return (
		<Dialog
			open={open}
			title={t('background.title')}
			size="medium"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton disabled={now === null} onClick={() => set(null)}>
						{t('background.remove')}
					</DialogButton>
					<DialogButton variant="primary" onClick={onClose}>
						{t('background.done')}
					</DialogButton>
				</DialogActions>
			}
		>
			<div className={styles.body}>
				<Segmented
					label={t('background.scope')}
					value={scope.id}
					options={scopes.map((s) => ({ value: s.id, label: t(s.label) }))}
					onChange={(id) => {
						const next = scopes.find((s) => s.id === id);
						if (!next) return;
						setChosen(next);
						setKind(kindOf(currentBackground(book, spread, next)));
					}}
				/>
				<Segmented
					label={t('background.kind')}
					value={kind}
					options={(['colour', 'gradient'] as const).map((k) => ({
						value: k,
						label: t(`background.kind.${k}`),
					}))}
					onChange={setKind}
				/>
				{kind === 'colour' ? (
					<ColourPicker
						label={t('background.colour')}
						value={now?.type === 'colour' ? now.colour : null}
						swatches={presets?.swatches ?? []}
						allowNone
						onPick={(colour) => set(colour === null ? null : { type: 'colour', colour })}
					/>
				) : (
					<div className={styles.gradients} role="group" aria-label={t('background.gradient')}>
						{(presets?.gradients ?? []).map((g) => {
							const bg: Extract<Background, { type: 'gradient' }> = {
								type: 'gradient',
								angle: g.angle,
								stops: g.stops,
							};
							const same =
								now?.type === 'gradient' &&
								now.angle === g.angle &&
								JSON.stringify(now.stops) === JSON.stringify(g.stops);
							return (
								<button
									key={g.name}
									type="button"
									className={styles.gradient}
									style={{ background: css(bg) }}
									aria-label={g.name}
									aria-pressed={same}
									title={g.name}
									onClick={() => set(bg)}
								/>
							);
						})}
					</div>
				)}
			</div>
		</Dialog>
	);
}
