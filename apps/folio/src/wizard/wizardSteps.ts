// The steps of the New book wizard, in the order a person meets them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** One step of the wizard: `id` keys its label and its page. */
export const WIZARD_STEPS = ['kind', 'size', 'title', 'words', 'way', 'look', 'review'] as const;

export type WizardStep = (typeof WIZARD_STEPS)[number];
