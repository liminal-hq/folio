// Jax's panel: a tip for what you are doing, a checklist for this page, and a guide through the book
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { PreflightItem } from '@liminal-hq/folio-protocol/generated/PreflightItem';
import { useState } from 'react';
import { describe } from '../app/preflightMessages';
import { t } from '../i18n/messages';
import {
	contextualTip,
	FIXED_GUIDE,
	FLOWING_GUIDE,
	GENERAL_TIPS,
	type GuideAction,
	type TipContext,
} from './jaxTips';
import styles from './JaxPanel.module.css';

type Tab = 'tip' | 'page' | 'guide';

interface JaxPanelProps {
	/** What the person is doing, for the tip. */
	context: TipContext;
	/** The issues the spread on show has, and what to call it on the shelf. */
	issues: readonly PreflightItem[];
	spreadName: string;
	/** Whether the person asked to be walked through, which opens the Guide. */
	guided: boolean;
	/** Does what "Show me" asks. */
	onShow(action: GuideAction): void;
	/** Puts Jax away. */
	onHush(): void;
}

export function JaxPanel({ context, issues, spreadName, guided, onShow, onHush }: JaxPanelProps) {
	const [tab, setTab] = useState<Tab>(guided ? 'guide' : 'tip');
	const [pool, setPool] = useState(0);
	const [step, setStep] = useState(0);
	const guide = context.flowing ? FLOWING_GUIDE : FIXED_GUIDE;
	const open = issues.filter((i) => i.severity !== 'note' && !i.ignored);
	const tip = contextualTip(context) ?? GENERAL_TIPS[pool % GENERAL_TIPS.length]!;
	const current = guide[Math.min(step, guide.length - 1)]!;

	/** "I'm lost": the first thing open on this page, else the next step of the guide, else what usually comes next. */
	const lost = () => {
		if (open.length > 0) setTab('page');
		else if (guided && step < guide.length - 1) {
			setStep(step + 1);
			setTab('guide');
		} else {
			setTab('tip');
			setPool((n) => n + 1);
		}
	};

	const tabs: Tab[] = guided ? ['tip', 'page', 'guide'] : ['tip', 'page'];
	return (
		<aside className={styles.panel} aria-label={t('jax.title')}>
			<div className={styles.head}>
				<span className={styles.face} aria-hidden="true">
					🦦
				</span>
				<div role="tablist" aria-label={t('jax.title')} className={styles.tabs}>
					{tabs.map((id) => (
						<button
							key={id}
							type="button"
							role="tab"
							aria-selected={tab === id}
							className={styles.tab}
							onClick={() => setTab(id)}
						>
							{t(`jax.tab.${id}`)}
						</button>
					))}
				</div>
				<button type="button" className={styles.hush} onClick={onHush}>
					{t('jax.hush')}
				</button>
			</div>
			<div role="tabpanel" className={styles.body}>
				{tab === 'tip' && (
					<>
						<p className={styles.says}>{t(tip)}</p>
						<div className={styles.actions}>
							<button type="button" className={styles.button} onClick={() => setPool((n) => n + 1)}>
								{t('jax.another')}
							</button>
							<button type="button" className={styles.button} onClick={lost}>
								{t('jax.lost')}
							</button>
						</div>
					</>
				)}
				{tab === 'page' &&
					(open.length === 0 ? (
						<p className={styles.says}>{t('jax.page.clear')}</p>
					) : (
						<>
							<p className={styles.hint}>{t('jax.page.heading', { where: spreadName })}</p>
							<ul className={styles.list}>
								{open.map((item) => (
									<li key={item.key || `${item.kind}:${item.frame ?? ''}:${item.page ?? ''}`}>
										{describe(item, spreadName).problem}
									</li>
								))}
							</ul>
						</>
					))}
				{tab === 'guide' && (
					<>
						<p className={styles.hint}>
							{t('jax.guide.step', { n: step + 1, of: guide.length })} · {t(current.title)}
						</p>
						<p className={styles.says}>{t(current.body)}</p>
						<div className={styles.actions}>
							<button
								type="button"
								className={styles.button}
								disabled={step === 0}
								onClick={() => setStep(step - 1)}
							>
								{t('jax.guide.back')}
							</button>
							{current.action.kind !== 'none' && (
								<button
									type="button"
									className={styles.button}
									onClick={() => onShow(current.action)}
								>
									{t('jax.guide.show')}
								</button>
							)}
							<button
								type="button"
								className={styles.button}
								disabled={step >= guide.length - 1}
								onClick={() => setStep(step + 1)}
							>
								{t('jax.guide.next')}
							</button>
						</div>
					</>
				)}
			</div>
		</aside>
	);
}
