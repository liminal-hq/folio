// The Book tools window's body: an area switcher, that area's tools, and the tool being used
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useState } from 'react';
import { t } from '../i18n/messages';
import styles from './ToolShell.module.css';
import { toolsIn } from './registry';
import { AREAS, LATER_AREAS, type ToolArea, type ToolProps } from './tools';

interface ToolShellProps {
	area: ToolArea;
	onArea(area: ToolArea): void;
	/** What the tools work on; `null` when no book is open. */
	work: Omit<ToolProps, 'say'> | null;
	/** The title and kind of the book, for the line under the switcher. */
	about: string | null;
	/** The id of the tool to show in its area, such as `structure.contents`: where the window opens. */
	tool?: string | null;
	/** Told which tool the person picks, so a later request for the same one is a change. */
	onTool?(tool: string): void;
}

/** The area a tool id belongs to: what comes before its first dot. */
const areaOf = (tool: string) => tool.split('.')[0]!;

export function ToolShell({ area, onArea, work, about, tool: asked, onTool }: ToolShellProps) {
	const tools = toolsIn(area);
	const [chosen, setChosen] = useState<Record<string, string>>(() =>
		asked ? { [areaOf(asked)]: asked } : {},
	);
	useEffect(() => {
		if (asked) setChosen((now) => ({ ...now, [areaOf(asked)]: asked }));
	}, [asked]);
	const [message, setMessage] = useState('');
	const current = tools.find((tool) => tool.id === chosen[area]) ?? tools[0] ?? null;

	return (
		<div className={styles.shell}>
			<nav className={styles.side} aria-label={t('tools.nav')}>
				<div className={styles.areas} role="tablist" aria-label={t('tools.areas')}>
					{AREAS.map((name) => (
						<button
							key={name}
							type="button"
							role="tab"
							aria-selected={name === area}
							className={styles.area}
							onClick={() => onArea(name)}
						>
							{t(`tools.area.${name}`)}
						</button>
					))}
					{LATER_AREAS.map((name) => (
						<button
							key={name}
							type="button"
							role="tab"
							aria-selected={false}
							disabled
							title={t('tools.later')}
							className={styles.area}
						>
							{t(`tools.area.${name}`)}
						</button>
					))}
				</div>
				{about && <p className={styles.about}>{about}</p>}
				<ul className={styles.tools}>
					{tools.map((tool) => (
						<li key={tool.id}>
							<button
								type="button"
								className={styles.tool}
								aria-current={tool.id === current?.id ? 'true' : undefined}
								onClick={() => {
									setChosen({ ...chosen, [area]: tool.id });
									onTool?.(tool.id);
								}}
							>
								{t(tool.titleKey)}
							</button>
						</li>
					))}
				</ul>
			</nav>
			<main className={styles.main}>
				{work === null ? (
					<p className={styles.empty}>{t('tools.noBook')}</p>
				) : current === null ? (
					<p className={styles.empty}>{t('tools.noTools')}</p>
				) : (
					<>
						<header className={styles.header}>
							<h1 className={styles.title}>{t(current.titleKey)}</h1>
							<p className={styles.desc}>{t(current.descKey)}</p>
						</header>
						<div className={styles.body}>
							<current.Component {...work} say={setMessage} />
						</div>
					</>
				)}
				<p className={styles.status} role="status">
					{message}
				</p>
			</main>
		</div>
	);
}
