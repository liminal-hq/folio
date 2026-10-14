// Supplies the window chrome's labels from the message catalogue
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { defaultChromeLabels, type ChromeLabels } from '@liminal-hq/folio-chrome/labels';

/** The chrome's labels. English only until the catalogue grows translations. */
export function chromeLabels(): ChromeLabels {
	return defaultChromeLabels;
}
