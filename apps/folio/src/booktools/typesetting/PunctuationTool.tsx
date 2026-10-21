// Smart punctuation: curly quotes, proper dashes and ellipses, with what would change shown first
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { PunctuationRule } from '@liminal-hq/folio-protocol/generated/PunctuationRule';
import { useEffect, useState } from 'react';
import { t } from '../../i18n/messages';
import type { ToolProps } from '../tools';
import { useToolReport } from '../useToolReport';
import styles from './typesetting.module.css';

export function PunctuationTool({ client, id, revision, say }: ToolProps) {
	const report = useToolReport(client, id, { tool: 'punctuation' }, revision);
	const found = report?.found ?? [];
	// Every rule that has something to do starts ticked.
	const [off, setOff] = useState<ReadonlySet<PunctuationRule>>(new Set());
	const [working, setWorking] = useState(false);
	useEffect(
		() => setOff((now) => new Set([...now].filter((r) => found.some((f) => f.rule === r)))),
		[report],
	);

	if (report === null) return <p className={styles.note}>{t('tools.working')}</p>;
	if (found.length === 0) return <p className={styles.note}>{t('tools.punctuation.none')}</p>;
	const chosen = found.filter((f) => !off.has(f.rule));
	const total = chosen.reduce((sum, f) => sum + f.count, 0);
	const fix = async () => {
		setWorking(true);
		try {
			const done = await client.bookToolFix(id, {
				tool: 'punctuation',
				rules: chosen.map((f) => f.rule),
			});
			say(t('tools.punctuation.fixed', { count: done.changed }));
		} catch {
			say(t('tools.punctuation.notFixed'));
		} finally {
			setWorking(false);
		}
	};
	return (
		<div>
			<ul className={styles.rules}>
				{found.map((rule) => (
					<li key={rule.rule} className={styles.rule}>
						<label className={styles.ruleHead}>
							<input
								type="checkbox"
								checked={!off.has(rule.rule)}
								onChange={(event) => {
									const next = new Set(off);
									if (event.target.checked) next.delete(rule.rule);
									else next.add(rule.rule);
									setOff(next);
								}}
							/>
							<span className={styles.ruleName}>{t(`tools.punctuation.rule.${rule.rule}`)}</span>
							<span className={styles.count}>
								{t('tools.punctuation.count', { count: rule.count })}
							</span>
						</label>
						{rule.examples.map((example, index) => (
							<p key={index} className={styles.diff}>
								<span className={styles.before}>{example.before}</span>
								<span aria-hidden="true"> → </span>
								<span className={styles.after}>{example.after}</span>
							</p>
						))}
					</li>
				))}
			</ul>
			<button
				type="button"
				className={styles.primary}
				disabled={working || chosen.length === 0}
				onClick={() => void fix()}
			>
				{t('tools.punctuation.fix', { count: total })}
			</button>
		</div>
	);
}
