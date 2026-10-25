// Keywords: up to seven phrases a store uses to help readers find the book
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useState } from 'react';
import { t } from '../../i18n/messages';
import type { ToolProps } from '../tools';
import styles from './publishing.module.css';

const SLOTS = 7;
const LONGEST = 50;

export function KeywordsTool({ book, apply, say }: ToolProps) {
	const kept = book.metadata.keywords;
	const [slots, setSlots] = useState(() => Array.from({ length: SLOTS }, (_, i) => kept[i] ?? ''));
	// A change made elsewhere (an undo, an agent) is shown, unless it is what was just typed here.
	useEffect(() => {
		setSlots((now) => {
			const typed = now.map((s) => s.trim()).filter((s) => s !== '');
			return JSON.stringify(typed) === JSON.stringify(kept)
				? now
				: Array.from({ length: SLOTS }, (_, i) => kept[i] ?? '');
		});
	}, [kept]);
	const change = (index: number, text: string) => {
		const next = slots.map((s, i) => (i === index ? text : s));
		setSlots(next);
		const keywords = next.map((s) => s.trim()).filter((s) => s !== '');
		void apply({ type: 'SetKeywords', keywords });
		say(t('tools.keywords.kept', { count: keywords.length }));
	};
	return (
		<div>
			<p className={styles.summary}>{t('tools.keywords.tip')}</p>
			<ul className={styles.list}>
				{slots.map((slot, index) => (
					<li key={index} className={styles.keyword}>
						<label className={styles.keywordField}>
							<span>{t('tools.keywords.slot', { n: index + 1 })}</span>
							<input
								type="text"
								value={slot}
								maxLength={LONGEST}
								onChange={(event) => change(index, event.target.value)}
							/>
						</label>
						<span className={styles.counter} aria-hidden="true">
							{slot.length}/{LONGEST}
						</span>
					</li>
				))}
			</ul>
		</div>
	);
}
