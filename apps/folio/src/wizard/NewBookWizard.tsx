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

export function NewBookWizard() {
	const controls = useWindowControls();
	const [index, setIndex] = useState(0);
	const step = WIZARD_STEPS[index] as WizardStep;
	const last = index === WIZARD_STEPS.length - 1;
	return (
		<WindowFrame>
			<div className={styles.screen}>
				<AppTitleBar title={t('window.wizardNew.title')} closeOnly />
				<div className={styles.body}>
					<nav aria-label={t('wizard.steps')} className={styles.steps}>
						<ol className={styles.list}>
							{WIZARD_STEPS.map((id, at) => (
								<li
									key={id}
									className={styles.step}
									aria-current={at === index ? 'step' : undefined}
									data-done={at < index || undefined}
								>
									<span className={styles.number}>{at + 1}</span>
									{LABELS[id]}
								</li>
							))}
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
						onClick={() => setIndex(index - 1)}
					>
						{t('wizard.back')}
					</button>
					<button
						type="button"
						className={styles.primary}
						disabled={last}
						onClick={() => setIndex(index + 1)}
					>
						{t('wizard.next')}
					</button>
				</footer>
			</div>
		</WindowFrame>
	);
}
