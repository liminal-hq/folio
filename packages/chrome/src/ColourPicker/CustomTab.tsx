// The Custom tab: the square, the hue slider, a hex field, RGB fields in Expert, and the eyedropper
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useId, useRef, useState } from 'react';
import {
	clamp,
	contrastGrade,
	contrastRatio,
	hexToRgb,
	hsvToRgb,
	parseColour,
	rgbToHex,
	rgbToHsv,
	type Hsv,
} from './colourMaths';
import styles from './ColourPicker.module.css';
import { SaturationSquare } from './SaturationSquare';
import type { ColourContrast, ColourPickerLabels } from './types';

/** The sampling API a webview may or may not have; WebKitGTK, which Linux builds use, does not. */
interface EyeDropperLike {
	open(): Promise<{ sRGBHex: string }>;
}
type EyeDropperConstructor = new () => EyeDropperLike;

const eyeDropper = (): EyeDropperConstructor | undefined =>
	(globalThis as { EyeDropper?: EyeDropperConstructor }).EyeDropper;

export interface CustomTabProps {
	/** The colour being edited, `#rrggbb`. */
	hex: string;
	labels: ColourPickerLabels;
	mode: 'easy' | 'expert';
	pageColour?: string;
	contrast?: ColourContrast;
	disabled?: boolean;
	/** A colour meant: the end of a drag, a key, or a field committed. */
	onApply(hex: string, final: boolean): void;
}

export function CustomTab(props: CustomTabProps) {
	const { hex, labels, mode, pageColour, contrast, disabled, onApply } = props;
	const [hsv, setHsv] = useState<Hsv>(() => rgbToHsv(hexToRgb(hex)));
	const [hexDraft, setHexDraft] = useState<string | null>(null);
	const [channelDraft, setChannelDraft] = useState<(string | null)[]>([null, null, null]);
	const [eyeNote, setEyeNote] = useState<string | null>(null);
	const ids = [useId(), useId(), useId(), useId()];
	const lastHex = useRef(hex);

	// The colour changed from outside (a swatch, a reset): follow it, keeping the hue of a grey.
	useEffect(() => {
		if (hex === lastHex.current) return;
		lastHex.current = hex;
		setHsv((prev) => {
			const next = rgbToHsv(hexToRgb(hex));
			return next.s === 0 || next.v === 0 ? { ...next, h: prev.h } : next;
		});
	}, [hex]);

	const shown = rgbToHex(hsvToRgb(hsv));
	const invalid = hexDraft !== null && hexDraft.trim() !== '' && parseColour(hexDraft) === null;
	const EyeDropper = eyeDropper();

	const apply = (next: string, final: boolean) => {
		lastHex.current = next;
		onApply(next, final);
	};
	const commitHex = (final: boolean) => {
		if (hexDraft === null) return;
		const parsed = parseColour(hexDraft);
		if (parsed === null) {
			if (final) return;
			setHexDraft(null);
			return;
		}
		setHexDraft(null);
		setHsv(rgbToHsv(hexToRgb(parsed)));
		apply(parsed, final);
	};
	const commitChannels = () => {
		if (channelDraft.every((c) => c === null)) return;
		const current = hexToRgb(shown);
		const next = current.map((value, i) => {
			const typed = channelDraft[i];
			const n = typed === null || typed === undefined ? NaN : Number(typed);
			return Number.isFinite(n) ? clamp(Math.round(n), 0, 255) : value;
		}) as [number, number, number];
		setChannelDraft([null, null, null]);
		setHsv(rgbToHsv(next));
		apply(rgbToHex(next), false);
	};

	const ratio = pageColour && contrast ? contrastRatio(shown, pageColour) : null;
	const grade = ratio === null ? null : contrastGrade(ratio);
	const gradeText =
		ratio === null
			? ''
			: contrast === 'graphic'
				? ratio >= 3
					? labels.contrastGraphicOk
					: labels.contrastGraphicLow
				: {
						aaa: labels.contrastAaa,
						aa: labels.contrastAa,
						large: labels.contrastLarge,
						fail: labels.contrastFail,
					}[grade!];

	return (
		<div className={styles.custom}>
			<SaturationSquare
				hue={hsv.h}
				saturation={hsv.s}
				value={hsv.v}
				label={labels.saturationBrightness}
				valueText={labels.saturationBrightnessValue(
					Math.round(hsv.s * 100),
					Math.round(hsv.v * 100),
				)}
				disabled={disabled}
				onChange={(s, v) => setHsv((prev) => ({ ...prev, s, v }))}
				onCommit={() => apply(shown, false)}
			/>
			<input
				type="range"
				className={styles.hue}
				min={0}
				max={360}
				step={1}
				value={Math.round(hsv.h)}
				aria-label={labels.hue}
				aria-valuetext={labels.hueValue(Math.round(hsv.h))}
				disabled={disabled}
				onChange={(event) => setHsv((prev) => ({ ...prev, h: Number(event.target.value) }))}
				onPointerUp={() => apply(shown, false)}
				onKeyUp={() => apply(shown, false)}
			/>
			<div className={styles.field}>
				<label htmlFor={ids[0]}>{labels.hex}</label>
				<input
					id={ids[0]}
					type="text"
					className={styles.input}
					value={hexDraft ?? shown}
					spellCheck={false}
					autoComplete="off"
					disabled={disabled}
					aria-invalid={invalid || undefined}
					aria-describedby={invalid ? `${ids[0]}-error` : undefined}
					onChange={(event) => setHexDraft(event.target.value)}
					onBlur={() => commitHex(false)}
					onKeyDown={(event) => {
						if (event.key !== 'Enter') return;
						event.preventDefault();
						commitHex(true);
					}}
				/>
				{invalid && (
					<p id={`${ids[0]}-error`} className={styles.error} role="alert">
						{labels.hexInvalid}
					</p>
				)}
			</div>
			{mode === 'expert' && (
				<div className={styles.channels} role="group">
					{[labels.red, labels.green, labels.blue].map((name, i) => (
						<label key={name} className={styles.channel}>
							<span>{name}</span>
							<input
								type="number"
								className={styles.input}
								min={0}
								max={255}
								step={1}
								inputMode="numeric"
								disabled={disabled}
								value={channelDraft[i] ?? hexToRgb(shown)[i]!}
								onChange={(event) =>
									setChannelDraft((prev) => prev.map((c, j) => (j === i ? event.target.value : c)))
								}
								onBlur={commitChannels}
								onKeyDown={(event) => {
									if (event.key !== 'Enter') return;
									event.preventDefault();
									commitChannels();
								}}
							/>
						</label>
					))}
				</div>
			)}
			<div className={styles.eye}>
				<button
					type="button"
					className={styles.button}
					style={{ justifySelf: 'start' }}
					disabled={disabled || !EyeDropper}
					aria-describedby={`${ids[1]}-eye`}
					onMouseDown={(event) => event.preventDefault()}
					onClick={() => {
						if (!EyeDropper) return;
						setEyeNote(null);
						new EyeDropper()
							.open()
							.then((result) => {
								const parsed = parseColour(result.sRGBHex);
								if (!parsed) return;
								setHsv(rgbToHsv(hexToRgb(parsed)));
								apply(parsed, true);
							})
							.catch(() => setEyeNote(labels.eyedropperCancelled));
					}}
				>
					{labels.eyedropper}
				</button>
				{(!EyeDropper || eyeNote) && (
					<p id={`${ids[1]}-eye`} className={styles.note}>
						{eyeNote ?? labels.eyedropperUnavailable}
					</p>
				)}
			</div>
			{ratio !== null && pageColour && (
				<div className={styles.contrast} data-grade={grade}>
					<span className={styles.sample} style={{ color: shown, background: pageColour }}>
						Aa
					</span>
					<span className={styles.contrastText}>
						<b>{labels.ratio(ratio.toFixed(2))}</b> · {gradeText}
						<br />
						{contrast === 'graphic' ? labels.contrastGraphic : labels.contrastText}
					</span>
				</div>
			)}
		</div>
	);
}
