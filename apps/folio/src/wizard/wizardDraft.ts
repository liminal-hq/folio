// What the wizard has been told so far, and the defaults a step starts from
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { NewBookSpec } from '@liminal-hq/folio-protocol/generated/NewBookSpec';

/** The book the wizard will make: every question it asks has a field here. */
export type WizardDraft = NewBookSpec;

/** The answers a person gets by pressing Next all the way, matching the quick start. */
export function initialDraft(): WizardDraft {
	return {
		kind: 'picture',
		trimIndex: 0,
		paper: 'white',
		title: '',
		subtitle: '',
		author: '',
		isbn: '',
		words: 'samples',
		coverColour: '#5b2a4a',
		look: 'friendly',
	};
}
