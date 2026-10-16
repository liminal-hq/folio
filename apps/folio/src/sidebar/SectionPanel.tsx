// The section the spread being shown starts: its name, how its pages are numbered, where they start
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { Numbering } from '@liminal-hq/folio-protocol/generated/Numbering';
import type { Section } from '@liminal-hq/folio-protocol/generated/Section';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { useState } from 'react';
import { Segmented } from '../app/Segmented';
import { t } from '../i18n/messages';
import styles from './PagesPanel.module.css';

export interface SectionPanelProps {
	spread: Spread;
	/** What the section the spread falls in is called, when it does not start one. */
	inside: string;
	onApply(intent: Intent): void;
}

/** A box that keeps what was typed on Enter or when it is left, and puts it back on Escape. */
function CommitField({
	label,
	value,
	inputMode,
	onCommit,
}: {
	label: string;
	value: string;
	inputMode?: 'numeric';
	onCommit(text: string): void;
}) {
	const [draft, setDraft] = useState<string | null>(null);
	const finish = () => {
		if (draft === null) return;
		setDraft(null);
		if (draft !== value) onCommit(draft);
	};
	return (
		<label className={styles.field}>
			<span>{label}</span>
			<input
				type="text"
				inputMode={inputMode}
				value={draft ?? value}
				onChange={(event) => setDraft(event.target.value)}
				onBlur={finish}
				onKeyDown={(event) => {
					if (event.key === 'Enter') {
						event.preventDefault();
						finish();
					} else if (event.key === 'Escape' && draft !== null) {
						event.preventDefault();
						event.stopPropagation();
						setDraft(null);
					}
				}}
			/>
		</label>
	);
}

/**
 * A section starts at a spread and runs to the next that starts one. Here a person starts one at the
 * spread they are looking at, names it, picks Arabic, Roman or no numbers, and says whether counting
 * starts again. The cover cannot start one.
 */
export function SectionPanel({ spread, inside, onApply }: SectionPanelProps) {
	if (spread.kind === 'cover') return null;
	const section = spread.section ?? null;
	const set = (next: Section | null) =>
		onApply({ type: 'SetSection', spread: spread.id, section: next });
	return (
		<section className={styles.section} aria-labelledby="pages-section">
			<h2 id="pages-section" className={styles.heading}>
				{t('section.heading')}
			</h2>
			{section === null ? (
				<>
					<p className={styles.note}>
						{inside ? t('section.inside', { name: inside }) : t('section.none')}
					</p>
					<button
						type="button"
						className={styles.action}
						onClick={() => set({ name: t('section.new'), numbering: 'arabic', start: 1 })}
					>
						{t('section.start')}
					</button>
				</>
			) : (
				<>
					<CommitField
						label={t('section.name')}
						value={section.name}
						onCommit={(name) => set({ ...section, name })}
					/>
					<Segmented<Numbering>
						label={t('section.numbering')}
						value={section.numbering}
						options={[
							{ value: 'arabic', label: '1, 2, 3' },
							{ value: 'roman', label: 'i, ii, iii' },
							{ value: 'none', label: t('section.numbering.none') },
						]}
						onChange={(numbering) => set({ ...section, numbering })}
					/>
					<label className={styles.check}>
						<input
							type="checkbox"
							checked={section.start != null}
							onChange={(event) => set({ ...section, start: event.target.checked ? 1 : null })}
						/>
						{t('section.restart')}
					</label>
					{section.start != null && (
						<CommitField
							label={t('section.startAt')}
							value={String(section.start)}
							inputMode="numeric"
							onCommit={(text) => {
								const start = Number(text.trim());
								if (Number.isInteger(start) && start >= 1 && start <= 99999) {
									set({ ...section, start });
								}
							}}
						/>
					)}
					<button type="button" className={styles.action} onClick={() => set(null)}>
						{t('section.remove')}
					</button>
				</>
			)}
		</section>
	);
}
