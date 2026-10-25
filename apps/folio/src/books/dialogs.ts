// The system's open and save pickers, behind an interface so tests can stand in for them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { open, save } from '@tauri-apps/plugin-dialog';

/** Where the person says a book is, or should go. `null` means they cancelled. */
export interface FileDialogs {
	/** Asks which `.folio` file to open. */
	pickBook(): Promise<string | null>;
	/** Asks where to save, starting at `defaultPath`. */
	pickSavePath(defaultPath: string): Promise<string | null>;
	/** Asks which picture to bring in. */
	pickImage(): Promise<string | null>;
	/** Asks which Markdown or text file to bring in. */
	pickText(): Promise<string | null>;
}

const FILTERS = [{ name: 'Folio books', extensions: ['folio'] }];
const IMAGE_FILTERS = [
	{ name: 'Pictures', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg'] },
];

const TEXT_FILTERS = [{ name: 'Markdown and text', extensions: ['md', 'markdown', 'txt'] }];

export const tauriFileDialogs: FileDialogs = {
	async pickBook() {
		const picked = await open({ multiple: false, directory: false, filters: FILTERS });
		return typeof picked === 'string' ? picked : null;
	},
	async pickSavePath(defaultPath) {
		return save({ defaultPath, filters: FILTERS });
	},
	async pickText() {
		const picked = await open({ multiple: false, directory: false, filters: TEXT_FILTERS });
		return typeof picked === 'string' ? picked : null;
	},
	async pickImage() {
		const picked = await open({ multiple: false, directory: false, filters: IMAGE_FILTERS });
		return typeof picked === 'string' ? picked : null;
	},
};
