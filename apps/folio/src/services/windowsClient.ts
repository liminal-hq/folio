// Opens the app's other windows through the app's own `open_window` command
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { invoke } from '@tauri-apps/api/core';
import type { WindowKind } from '@liminal-hq/folio-protocol/generated/WindowKind';

/** Opens a window of a kind, or focuses it when it is already open. */
export interface WindowsClient {
	open(kind: WindowKind): Promise<void>;
}

/** The Rust `open_window` command, which the app itself registers (it is not a plugin command). */
export const tauriWindowsClient: WindowsClient = {
	open: (kind) => invoke<void>('open_window', { kind }),
};
