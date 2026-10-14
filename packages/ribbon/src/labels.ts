// Default English labels for the ribbon's own controls
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** User-visible strings the ribbon owns. Apps pass translated values through the `labels` prop. */
export interface RibbonLabels {
	/** Accessible name of the ribbon. */
	ribbon: string;
	/** Accessible name of the toolbar of controls inside the active tab's panel. */
	toolbar: string;
	/** Accessible name of the row of tabs. */
	tabs: string;
	/** Accessible name of the Easy and Expert switch. */
	mode: string;
	modeEasy: string;
	modeExpert: string;
	/** Keeps a collapsed ribbon open. */
	pin: string;
	unpin: string;
	/** Prefix of a stepper's accessible names: "Decrease {label}". */
	decrease: string;
	increase: string;
}

export const defaultRibbonLabels: RibbonLabels = {
	ribbon: 'Ribbon',
	toolbar: 'Ribbon controls',
	tabs: 'Ribbon tabs',
	mode: 'How do you like to work?',
	modeEasy: 'Easy',
	modeExpert: 'Expert',
	pin: 'Keep the ribbon open',
	unpin: 'Let the ribbon close',
	decrease: 'Decrease',
	increase: 'Increase',
};
