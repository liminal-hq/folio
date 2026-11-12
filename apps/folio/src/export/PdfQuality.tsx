// Picture quality for each kind of PDF: one choice, with a line saying what each is for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ExportPrefs } from '@liminal-hq/folio-protocol/generated/ExportPrefs';
import type { PdfPrefs } from '@liminal-hq/folio-protocol/generated/PdfPrefs';
import { t, type MessageKey } from '../i18n/messages';
import styles from './PdfQuality.module.css';
import { BUILTIN_PRESET, effectivePdf, PRESETS, type PdfKind } from './exportPrefs';

interface PdfQualityProps {
	kinds: PdfKind[];
	user: ExportPrefs;
	book: ExportPrefs;
	edits: ExportPrefs;
	onChange(kind: PdfKind, change: Partial<PdfPrefs>): void;
	/** Makes what was changed the person's defaults; left out where there is nowhere to keep them. */
	onMakeDefaults?: () => void;
	/** Whether anything was changed in this dialog. */
	changed: boolean;
}

export function PdfQuality({
	kinds,
	user,
	book,
	edits,
	onChange,
	onMakeDefaults,
	changed,
}: PdfQualityProps) {
	return (
		<section className={styles.quality} aria-label={t('export.quality')}>
			{kinds.map((kind) => {
				const preset = effectivePdf(kind, user, book, edits).preset ?? BUILTIN_PRESET[kind];
				const options = PRESETS[kind];
				// A press file offers only what keeps 300 pixels an inch; an older choice of less is
				// shown as the nearest it can be.
				const shown = options.includes(preset) ? preset : 'high';
				return (
					<fieldset key={kind} className={styles.group}>
						<legend className={styles.legend}>
							{kinds.length > 1
								? t('export.quality.for', { name: t(`export.quality.kind.${kind}` as MessageKey) })
								: t('export.quality')}
						</legend>
						{options.map((option) => (
							<label key={option} className={styles.option}>
								<input
									type="radio"
									name={`quality-${kind}`}
									checked={shown === option}
									onChange={() => onChange(kind, { preset: option })}
								/>
								<span className={styles.text}>
									<span className={styles.name}>{t(`export.quality.${option}` as MessageKey)}</span>
									<span className={styles.sub}>
										{t(`export.quality.${option}.sub` as MessageKey)}
									</span>
								</span>
							</label>
						))}
					</fieldset>
				);
			})}
			{onMakeDefaults && (
				<button
					type="button"
					className={styles.defaults}
					disabled={!changed}
					onClick={onMakeDefaults}
				>
					{t('export.quality.makeDefaults')}
				</button>
			)}
		</section>
	);
}
