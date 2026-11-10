// The Properties panel's Paragraph section: spacing, line spacing and indents of the paragraphs the ribbon's controls act on
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { ParaChange } from '@liminal-hq/folio-protocol/generated/ParaChange';
import type { ParaLook } from '@liminal-hq/folio-protocol/generated/ParaLook';
import { t } from '../i18n/messages';
import {
	FROM_STYLE,
	LINE_SPACINGS,
	MAX_INDENT,
	paragraphScopes,
	type ParagraphInput,
} from '../ribbon/paragraphCommands';
import styles from './Inspector.module.css';
import { LengthField } from './LengthField';
import type { Units } from './units';

export interface ParagraphSectionProps extends ParagraphInput {
	units: Units;
	disabled?: boolean;
	apply(intent: Intent): void;
}

const lineLabel = (n: number) => n.toFixed(2).replace(/0$/, '');

/**
 * Space before and after, line spacing, the indent and the first line's indent. They change the
 * paragraphs under the caret or selection of the open editor, or every paragraph of the frame
 * when no editor is open; each paragraph keeps whatever else it set.
 */
export function ParagraphSection(props: ParagraphSectionProps) {
	const { units, disabled, apply } = props;
	const scopes = paragraphScopes(props);
	if (scopes.length === 0) return null;
	const look: ParaLook = scopes[0]!.looks[0] ?? {};
	const across = scopes.some((s) => s.to > s.from);
	const send = (change: ParaChange) =>
		scopes.forEach((s) =>
			apply({ type: 'SetParagraphLook', target: s.target, from: s.from, to: s.to, change }),
		);
	const length = (
		field: 'spaceBefore' | 'spaceAfter' | 'indent' | 'firstIndent',
		label: string,
	) => (
		<LengthField
			label={label}
			points={look[field] ?? 0}
			units={units}
			disabled={disabled}
			onCommit={(points) =>
				points >= 0 && points <= MAX_INDENT && send({ set: { [field]: points }, clear: [] })
			}
		/>
	);
	return (
		<fieldset className={styles.lengths} aria-label={t('inspector.para')}>
			<legend>{t('inspector.para')}</legend>
			{length('spaceBefore', t('inspector.para.before'))}
			{length('spaceAfter', t('inspector.para.after'))}
			{length('indent', t('inspector.para.indent'))}
			{length('firstIndent', t('inspector.para.first'))}
			<label className={styles.length}>
				<span>{t('inspector.para.line')}</span>
				<select
					className={styles.select}
					value={look.lineHeight === undefined ? FROM_STYLE : String(look.lineHeight)}
					disabled={disabled}
					onChange={(event) =>
						send(
							event.target.value === FROM_STYLE
								? { set: {}, clear: ['lineHeight'] }
								: { set: { lineHeight: Number(event.target.value) }, clear: [] },
						)
					}
				>
					<option value={FROM_STYLE}>{t('ribbon.para.fromStyle')}</option>
					{look.lineHeight !== undefined && !LINE_SPACINGS.some((n) => n === look.lineHeight) && (
						<option value={String(look.lineHeight)}>{lineLabel(look.lineHeight)}</option>
					)}
					{LINE_SPACINGS.map((n) => (
						<option key={n} value={String(n)}>
							{lineLabel(n)}
						</option>
					))}
				</select>
			</label>
			<p className={styles.note} style={{ gridColumn: '1 / -1' }}>
				{t(across || !props.paragraphs ? 'inspector.para.scopeMany' : 'inspector.para.scopeOne')}
			</p>
		</fieldset>
	);
}
