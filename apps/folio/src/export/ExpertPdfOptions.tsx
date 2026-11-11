// The Expert panels of the Export dialog: pictures, navigation and information, pages, print extras
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ExportPrefs } from '@liminal-hq/folio-protocol/generated/ExportPrefs';
import type { PdfPrefs } from '@liminal-hq/folio-protocol/generated/PdfPrefs';
import { useId, useState, type ReactNode } from 'react';
import { t, type MessageKey } from '../i18n/messages';
import styles from './ExpertPdfOptions.module.css';
import {
	builtinPdf,
	effectivePdf,
	lockedBecause,
	PRESET_FACTS,
	type LockedField,
	type PdfKind,
} from './exportPrefs';

interface ExpertPdfOptionsProps {
	kinds: PdfKind[];
	user: ExportPrefs;
	book: ExportPrefs;
	edits: ExportPrefs;
	onChange(kind: PdfKind, change: Partial<PdfPrefs>): void;
}

interface RowProps {
	label: string;
	reason: MessageKey | null;
	children: (props: { id: string; disabled: boolean; describedBy?: string }) => ReactNode;
}

/** A labelled control that is greyed, with the reason beside it, when its destination fixes it. */
function Row({ label, reason, children }: RowProps) {
	const id = useId();
	const reasonId = `${id}-why`;
	return (
		<div className={styles.row} data-locked={reason ? 'true' : undefined}>
			<label htmlFor={id} className={styles.label}>
				{label}
			</label>
			<div className={styles.control}>
				{children({ id, disabled: reason !== null, describedBy: reason ? reasonId : undefined })}
				{reason && (
					<span id={reasonId} className={styles.why}>
						{t(reason)}
					</span>
				)}
			</div>
		</div>
	);
}

const number = (text: string): number | undefined => {
	const n = Number(text);
	return text.trim() === '' || !Number.isFinite(n) ? undefined : n;
};

export function ExpertPdfOptions({ kinds, user, book, edits, onChange }: ExpertPdfOptionsProps) {
	const [at, setAt] = useState<PdfKind>(kinds[0] ?? 'sharing');
	const kind = kinds.includes(at) ? at : (kinds[0] ?? 'sharing');
	const eff: PdfPrefs = {
		...builtinPdf(kind),
		...effectivePdf(kind, user, book, edits),
	};
	const set = (change: Partial<PdfPrefs>) => onChange(kind, change);
	const lock = (field: LockedField) => lockedBecause(kind, field);
	const facts = PRESET_FACTS[eff.preset ?? 'standard'];
	const press = kind !== 'sharing';
	const bleed = eff.bleed ?? { kind: 'book' };

	return (
		<details className={styles.expert}>
			<summary className={styles.summary}>{t('export.expert')}</summary>
			{kinds.length > 1 && (
				<div role="group" aria-label={t('export.expert.for')} className={styles.kinds}>
					{kinds.map((k) => (
						<button
							key={k}
							type="button"
							aria-pressed={k === kind}
							className={styles.kind}
							onClick={() => setAt(k)}
						>
							{t(`export.quality.kind.${k}` as MessageKey)}
						</button>
					))}
				</div>
			)}

			<fieldset className={styles.panel}>
				<legend className={styles.legend}>{t('export.expert.pictures')}</legend>
				<Row label={t('export.expert.ppi')} reason={null}>
					{({ id, disabled }) => (
						<input
							id={id}
							type="number"
							min={press ? 300 : 72}
							max={1200}
							disabled={disabled}
							value={eff.maxPpi ?? ''}
							placeholder={facts.ppi === null ? t('export.expert.noLimit') : String(facts.ppi)}
							onChange={(e) => set({ maxPpi: number(e.target.value) })}
						/>
					)}
				</Row>
				<Row label={t('export.expert.quality')} reason={null}>
					{({ id, disabled }) => (
						<input
							id={id}
							type="number"
							min={press ? 80 : 1}
							max={100}
							disabled={disabled}
							value={eff.jpegQuality ?? ''}
							placeholder={String(facts.quality)}
							onChange={(e) => set({ jpegQuality: number(e.target.value) })}
						/>
					)}
				</Row>
				<Row label={t('export.expert.compression')} reason={null}>
					{({ id, disabled }) => (
						<select
							id={id}
							disabled={disabled}
							value={eff.compression ?? 'auto'}
							onChange={(e) => set({ compression: e.target.value as PdfPrefs['compression'] })}
						>
							<option value="auto">{t('export.expert.compression.auto')}</option>
							<option value="jpeg">{t('export.expert.compression.jpeg')}</option>
							<option value="lossless">{t('export.expert.compression.lossless')}</option>
						</select>
					)}
				</Row>
				<Row label={t('export.expert.keepJpeg')} reason={null}>
					{({ id, disabled }) => (
						<input
							id={id}
							type="checkbox"
							disabled={disabled}
							checked={eff.keepJpeg === true}
							onChange={(e) => set({ keepJpeg: e.target.checked })}
						/>
					)}
				</Row>
			</fieldset>

			<fieldset className={styles.panel}>
				<legend className={styles.legend}>{t('export.expert.navigation')}</legend>
				<Row label={t('export.expert.bookmarks')} reason={lock('bookmarks')}>
					{({ id, disabled, describedBy }) => (
						<select
							id={id}
							disabled={disabled}
							aria-describedby={describedBy}
							value={disabled ? 'none' : (eff.bookmarks ?? 'none')}
							onChange={(e) => set({ bookmarks: e.target.value as PdfPrefs['bookmarks'] })}
						>
							<option value="none">{t('export.expert.bookmarks.none')}</option>
							<option value="chapters">{t('export.expert.bookmarks.chapters')}</option>
							<option value="spreads">{t('export.expert.bookmarks.spreads')}</option>
							<option value="pages">{t('export.expert.bookmarks.pages')}</option>
						</select>
					)}
				</Row>
				<Row label={t('export.expert.pageLabels')} reason={lock('pageLabels')}>
					{({ id, disabled, describedBy }) => (
						<input
							id={id}
							type="checkbox"
							disabled={disabled}
							aria-describedby={describedBy}
							checked={!disabled && eff.pageLabels === true}
							onChange={(e) => set({ pageLabels: e.target.checked })}
						/>
					)}
				</Row>
				<Row label={t('export.expert.links')} reason={lock('links') ?? 'export.locked.linksSoon'}>
					{({ id, disabled, describedBy }) => (
						<input
							id={id}
							type="checkbox"
							disabled={disabled}
							aria-describedby={describedBy}
							checked={false}
							readOnly
						/>
					)}
				</Row>
				<Row label={t('export.expert.viewer')} reason={lock('viewerLayout')}>
					{({ id, disabled, describedBy }) => (
						<select
							id={id}
							disabled={disabled}
							aria-describedby={describedBy}
							value={disabled ? 'single' : (eff.viewerLayout ?? 'single')}
							onChange={(e) => set({ viewerLayout: e.target.value as PdfPrefs['viewerLayout'] })}
						>
							<option value="single">{t('export.expert.viewer.single')}</option>
							<option value="spreads">{t('export.expert.viewer.spreads')}</option>
						</select>
					)}
				</Row>
				<Row label={t('export.expert.subject')} reason={lock('subject')}>
					{({ id, disabled, describedBy }) => (
						<input
							id={id}
							type="text"
							disabled={disabled}
							aria-describedby={describedBy}
							value={disabled ? '' : (eff.subject ?? '')}
							placeholder={t('export.expert.subject.hint')}
							onChange={(e) => set({ subject: e.target.value || undefined })}
						/>
					)}
				</Row>
				<Row label={t('export.expert.keywords')} reason={lock('keywords')}>
					{({ id, disabled, describedBy }) => (
						<input
							id={id}
							type="text"
							disabled={disabled}
							aria-describedby={describedBy}
							value={disabled ? '' : (eff.keywords ?? []).join(', ')}
							placeholder={t('export.expert.keywords.hint')}
							onChange={(e) => {
								const words = e.target.value
									.split(',')
									.map((w) => w.trim())
									.filter(Boolean);
								set({ keywords: words.length > 0 ? words : undefined });
							}}
						/>
					)}
				</Row>
			</fieldset>

			<fieldset className={styles.panel}>
				<legend className={styles.legend}>{t('export.expert.pages')}</legend>
				<Row label={t('export.expert.sheets')} reason={lock('sheets')}>
					{({ id, disabled, describedBy }) => (
						<select
							id={id}
							disabled={disabled}
							aria-describedby={describedBy}
							value={disabled ? 'pages' : (eff.sheets ?? 'pages')}
							onChange={(e) => set({ sheets: e.target.value as PdfPrefs['sheets'] })}
						>
							<option value="pages">{t('export.expert.sheets.pages')}</option>
							<option value="spreads">{t('export.expert.sheets.spreads')}</option>
						</select>
					)}
				</Row>
				<Row label={t('export.expert.cover')} reason={lock('cover')}>
					{({ id, disabled, describedBy }) => (
						<select
							id={id}
							disabled={disabled}
							aria-describedby={describedBy}
							value={disabled ? 'front' : (eff.cover ?? 'front')}
							onChange={(e) => set({ cover: e.target.value as PdfPrefs['cover'] })}
						>
							<option value="front">{t('export.expert.cover.front')}</option>
							<option value="frontAndBack">{t('export.expert.cover.both')}</option>
							<option value="none">{t('export.expert.cover.none')}</option>
						</select>
					)}
				</Row>
				<Row label={t('export.expert.rangeFrom')} reason={lock('range')}>
					{({ id, disabled, describedBy }) => (
						<input
							id={id}
							type="number"
							min={1}
							disabled={disabled}
							aria-describedby={describedBy}
							value={disabled ? '' : (eff.range?.[0] ?? '')}
							placeholder={t('export.expert.rangeAll')}
							onChange={(e) => {
								const first = number(e.target.value);
								set({
									range: first === undefined ? undefined : [first, eff.range?.[1] ?? first],
								});
							}}
						/>
					)}
				</Row>
				<Row label={t('export.expert.rangeTo')} reason={lock('range')}>
					{({ id, disabled, describedBy }) => (
						<input
							id={id}
							type="number"
							min={1}
							disabled={disabled || eff.range === undefined}
							aria-describedby={describedBy}
							value={disabled ? '' : (eff.range?.[1] ?? '')}
							onChange={(e) => {
								const last = number(e.target.value);
								if (eff.range && last !== undefined) set({ range: [eff.range[0], last] });
							}}
						/>
					)}
				</Row>
			</fieldset>

			<fieldset className={styles.panel}>
				<legend className={styles.legend}>{t('export.expert.print')}</legend>
				<Row label={t('export.expert.cropMarks')} reason={lock('cropMarks')}>
					{({ id, disabled, describedBy }) => (
						<input
							id={id}
							type="checkbox"
							disabled={disabled}
							aria-describedby={describedBy}
							checked={!disabled && eff.cropMarks === true}
							onChange={(e) => set({ cropMarks: e.target.checked })}
						/>
					)}
				</Row>
				<Row label={t('export.expert.bleed')} reason={lock('bleed')}>
					{({ id, disabled, describedBy }) => (
						<select
							id={id}
							disabled={disabled}
							aria-describedby={describedBy}
							value={disabled ? 'book' : bleed.kind}
							onChange={(e) =>
								set({
									bleed:
										e.target.value === 'custom' ? { kind: 'custom', points: 9 } : { kind: 'book' },
								})
							}
						>
							<option value="book">{t('export.expert.bleed.book')}</option>
							<option value="custom">{t('export.expert.bleed.custom')}</option>
						</select>
					)}
				</Row>
				<Row label={t('export.expert.bleedPoints')} reason={lock('bleed')}>
					{({ id, disabled, describedBy }) => (
						<input
							id={id}
							type="number"
							min={0}
							max={36}
							disabled={disabled || bleed.kind !== 'custom'}
							aria-describedby={describedBy}
							value={bleed.kind === 'custom' && !disabled ? bleed.points : ''}
							onChange={(e) => {
								const points = number(e.target.value);
								if (points !== undefined) set({ bleed: { kind: 'custom', points } });
							}}
						/>
					)}
				</Row>
				<Row label={t('export.expert.slug')} reason={lock('slug')}>
					{({ id, disabled, describedBy }) => (
						<input
							id={id}
							type="number"
							min={18}
							max={72}
							disabled={disabled}
							aria-describedby={describedBy}
							value={disabled ? '' : (eff.slug ?? '')}
							placeholder="24"
							onChange={(e) => set({ slug: number(e.target.value) })}
						/>
					)}
				</Row>
				<Row label={t('export.expert.ink')} reason={lock('ink')}>
					{({ id, disabled, describedBy }) => (
						<input
							id={id}
							type="checkbox"
							disabled={disabled}
							aria-describedby={describedBy}
							checked={!disabled && eff.ink === true}
							onChange={(e) => set({ ink: e.target.checked })}
						/>
					)}
				</Row>
				<Row label={t('export.expert.profile')} reason={lock('profile')}>
					{({ id, disabled, describedBy }) => (
						<input
							id={id}
							type="text"
							disabled={disabled || eff.ink !== true}
							aria-describedby={describedBy}
							value={disabled ? '' : (eff.profile ?? '')}
							placeholder={t('export.colour.profile.hint')}
							onChange={(e) => set({ profile: e.target.value || undefined })}
						/>
					)}
				</Row>
			</fieldset>
		</details>
	);
}
