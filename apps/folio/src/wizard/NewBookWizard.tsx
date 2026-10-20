// The New book wizard window: a step list, the step's page, and Back, Next and Skip
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { WindowFrame } from '@liminal-hq/folio-chrome/WindowFrame';
import { useWindowControls } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { useState } from 'react';
import { AppTitleBar } from '../app/AppTitleBar';
import { t } from '../i18n/messages';
import styles from './NewBookWizard.module.css';
import { WIZARD_STEPS, type WizardStep } from './wizardSteps';

const LABELS: Record<WizardStep, string> = {
	kind: t('wizard.step.kind'),
	size: t('wizard.step.size'),
	title: t('wizard.step.title'),
	words: t('wizard.step.words'),
	way: t('wizard.step.way'),
	look: t('wizard.step.look'),
	review: t('wizard.step.review'),
};

function Tick() {
	return (
		<svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
			<path
				d="M2.5 6.5l2.3 2.3L9.5 3.7"
				fill="none"
				stroke="currentColor"
				strokeWidth="1.8"
				strokeLinecap="round"
				strokeLinejoin="round"
			/>
		</svg>
	);
}

export function NewBookWizard() {
	const controls = useWindowControls();
	const [index, setIndex] = useState(0);
	// The furthest step met: a step already seen can be returned to by clicking it.
	const [reached, setReached] = useState(0);
	const go = (to: number) => {
		setIndex(to);
		setReached((far) => Math.max(far, to));
	};
	const step = WIZARD_STEPS[index] as WizardStep;
	const last = index === WIZARD_STEPS.length - 1;
	return (
		<WindowFrame>
			<div className={styles.screen}>
				<AppTitleBar title={t('window.wizardNew.title')} closeOnly />
				<div className={styles.body}>
					<nav aria-label={t('wizard.steps')} className={styles.steps}>
						<ol className={styles.list}>
							{WIZARD_STEPS.map((id, at) => {
								// A step met stays marked, wherever the person goes back to.
								const visited = at <= reached && at !== index;
								return (
									<li
										key={id}
										className={styles.item}
										aria-current={at === index ? 'step' : undefined}
										data-visited={visited || undefined}
									>
										<button
											type="button"
											className={styles.step}
											disabled={at > reached}
											onClick={() => go(at)}
										>
											<span className={styles.number} aria-hidden="true">
												{visited ? <Tick /> : at + 1}
											</span>
											<span className={styles.label} data-text={LABELS[id]}>
												{LABELS[id]}
											</span>
											{visited && <span className={styles.hidden}>{t('wizard.visited')}</span>}
										</button>
									</li>
								);
							})}
						</ol>
					</nav>
					<main className={styles.page}>
						<h1 className={styles.heading}>{LABELS[step]}</h1>
						<p className={styles.note}>{t('wizard.comingSoon')}</p>
					</main>
				</div>
				<footer className={styles.footer}>
					<button type="button" className={styles.skip} onClick={() => controls.close()}>
						{t('wizard.skip')}
					</button>
					<span className={styles.spacer} />
					<button
						type="button"
						className={styles.button}
						disabled={index === 0}
						onClick={() => go(index - 1)}
					>
						{t('wizard.back')}
					</button>
					<button
						type="button"
						className={styles.primary}
						disabled={last}
						onClick={() => go(index + 1)}
					>
						{t('wizard.next')}
					</button>
				</footer>
			</div>
		</WindowFrame>
	);
}
