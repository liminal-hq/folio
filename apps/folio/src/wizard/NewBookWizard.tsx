// The New book wizard window: a step list, the step's page, and Back, Next and Skip
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { WindowFrame } from '@liminal-hq/folio-chrome/WindowFrame';
import { useWindowControls } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import type { KindInfo } from '@liminal-hq/folio-plugin-books';
import { useEffect, useState } from 'react';
import { AppTitleBar } from '../app/AppTitleBar';
import type { BooksClient } from '../books/booksClient';
import { tauriBooksClient } from '../books/booksClient';
import { t } from '../i18n/messages';
import styles from './NewBookWizard.module.css';
import { KindStep } from './KindStep';
import { SizeStep } from './SizeStep';
import { useSettings, useUpdateSettings } from '../settings/useSettings';
import { LookStep } from './LookStep';
import { ReviewStep } from './ReviewStep';
import { TitleStep } from './TitleStep';
import { WayStep } from './WayStep';
import { WordsStep } from './WordsStep';
import { initialDraft, type WizardDraft } from './wizardDraft';
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

export function NewBookWizard({ client = tauriBooksClient }: { client?: BooksClient }) {
	const controls = useWindowControls();
	const settings = useSettings();
	const update = useUpdateSettings();
	const [making, setMaking] = useState(false);
	const [problem, setProblem] = useState<string | null>(null);
	const [kinds, setKinds] = useState<KindInfo[]>([]);
	const [draft, setDraft] = useState<WizardDraft>(() => initialDraft(settings));
	const change = (patch: Partial<WizardDraft>) => setDraft((was) => ({ ...was, ...patch }));
	useEffect(() => {
		let live = true;
		client
			.presets()
			.then((found) => live && setKinds(found))
			.catch(() => {});
		return () => {
			live = false;
		};
	}, [client]);
	const [index, setIndex] = useState(0);
	// The furthest step met: a step already seen can be returned to by clicking it.
	const [reached, setReached] = useState(0);
	const go = (to: number) => {
		setIndex(to);
		setReached((far) => Math.max(far, to));
	};
	const make = async () => {
		setMaking(true);
		setProblem(null);
		try {
			const { mode, jax, ...spec } = draft;
			// The way of working is the person's setting, so it is kept for the next book too.
			await update({ mode, jax });
			await client.newBook({
				...spec,
				title: spec.title.trim(),
				subtitle: spec.subtitle.trim(),
				author: spec.author.trim(),
				isbn: spec.isbn.trim(),
			});
			controls.close();
		} catch (reason) {
			const message =
				reason instanceof Error
					? reason.message
					: typeof reason === 'object' && reason !== null && 'message' in reason
						? String((reason as { message: unknown }).message)
						: String(reason);
			setProblem(t('wizard.makeFailed', { message }));
			setMaking(false);
		}
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
						{step === 'kind' && <KindStep kinds={kinds} draft={draft} onChange={change} />}
						{step === 'size' && (
							<SizeStep
								kind={kinds.find((info) => info.kind === draft.kind)}
								draft={draft}
								onChange={change}
							/>
						)}
						{step === 'title' && <TitleStep draft={draft} onChange={change} />}
						{step === 'words' && <WordsStep draft={draft} onChange={change} />}
						{step === 'way' && <WayStep draft={draft} onChange={change} />}
						{step === 'look' && <LookStep draft={draft} onChange={change} />}
						{step === 'review' && (
							<ReviewStep
								kind={kinds.find((info) => info.kind === draft.kind)}
								draft={draft}
								problem={problem}
							/>
						)}
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
					{last ? (
						<button type="button" className={styles.primary} disabled={making} onClick={make}>
							{making ? t('wizard.making') : t('wizard.make')}
						</button>
					) : (
						<button type="button" className={styles.primary} onClick={() => go(index + 1)}>
							{t('wizard.next')}
						</button>
					)}
				</footer>
			</div>
		</WindowFrame>
	);
}
