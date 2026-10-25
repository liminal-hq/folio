// Print cost and royalty: what a copy costs to print and what is earned on a sale
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useState } from 'react';
import { t } from '../../i18n/messages';
import type { ToolProps } from '../tools';
import { useToolReport } from '../useToolReport';
import styles from './publishing.module.css';

const money = (value: number) => `$${value.toFixed(2)}`;
const number = (text: string) => {
	const value = Number.parseFloat(text);
	return Number.isFinite(value) && value >= 0 ? value : 0;
};

export function RoyaltyTool({ client, id, revision, expert }: ToolProps) {
	const [hardcover, setHardcover] = useState(false);
	const [colour, setColour] = useState(false);
	const [list, setList] = useState('12.99');
	const [ebook, setEbook] = useState('4.99');
	const [fixed, setFixed] = useState('');
	const [perPage, setPerPage] = useState('');
	const report = useToolReport(
		client,
		id,
		{
			tool: 'royalty',
			hardcover,
			colour,
			listPrice: number(list),
			ebookPrice: number(ebook),
			fixedCost: fixed.trim() === '' ? null : number(fixed),
			perPageCost: perPage.trim() === '' ? null : number(perPage),
		},
		revision,
	);
	return (
		<div className={styles.royalty}>
			<div className={styles.controls}>
				<fieldset className={styles.group}>
					<legend>{t('tools.royalty.format')}</legend>
					<label>
						<input type="radio" checked={!hardcover} onChange={() => setHardcover(false)} />
						{t('tools.royalty.paperback')}
					</label>
					<label>
						<input type="radio" checked={hardcover} onChange={() => setHardcover(true)} />
						{t('tools.royalty.hardcover')}
					</label>
				</fieldset>
				<fieldset className={styles.group}>
					<legend>{t('tools.royalty.ink')}</legend>
					<label>
						<input type="radio" checked={!colour} onChange={() => setColour(false)} />
						{t('tools.royalty.black')}
					</label>
					<label>
						<input type="radio" checked={colour} onChange={() => setColour(true)} />
						{t('tools.royalty.colour')}
					</label>
				</fieldset>
				<label className={styles.field}>
					<span>{t('tools.royalty.list')}</span>
					<input
						type="number"
						min="0"
						step="0.01"
						value={list}
						onChange={(event) => setList(event.target.value)}
					/>
				</label>
				<label className={styles.field}>
					<span>{t('tools.royalty.ebook')}</span>
					<input
						type="number"
						min="0"
						step="0.01"
						value={ebook}
						onChange={(event) => setEbook(event.target.value)}
					/>
				</label>
				{expert && (
					<>
						<label className={styles.field}>
							<span>{t('tools.royalty.fixed')}</span>
							<input
								type="number"
								min="0"
								step="0.01"
								value={fixed}
								placeholder={report ? report.costs.fixed.toFixed(2) : ''}
								onChange={(event) => setFixed(event.target.value)}
							/>
						</label>
						<label className={styles.field}>
							<span>{t('tools.royalty.perPage')}</span>
							<input
								type="number"
								min="0"
								step="0.001"
								value={perPage}
								placeholder={report ? report.costs.perPage.toFixed(3) : ''}
								onChange={(event) => setPerPage(event.target.value)}
							/>
						</label>
					</>
				)}
			</div>
			<div className={styles.results} aria-live="polite">
				{report === null ? (
					<p className={styles.note}>{t('tools.working')}</p>
				) : (
					<>
						<p className={styles.big}>
							{t('tools.royalty.printCost', {
								cost: money(report.printCost),
								pages: report.pages,
							})}
						</p>
						<table className={styles.table}>
							<tbody>
								{report.sales.map((sale) => (
									<tr key={sale.market}>
										<th scope="row">{t(`tools.royalty.market.${sale.market}`)}</th>
										<td>{money(sale.royalty)}</td>
									</tr>
								))}
								<tr>
									<th scope="row">{t('tools.royalty.ebookEarned')}</th>
									<td>{money(report.ebookRoyalty)}</td>
								</tr>
							</tbody>
						</table>
						<p className={styles.hint}>
							{t('tools.royalty.minimum', { price: money(report.minimumPrice) })}
						</p>
						<p className={styles.warning}>
							{t('tools.royalty.estimate', { date: report.costs.asOf })}
						</p>
					</>
				)}
			</div>
		</div>
	);
}
