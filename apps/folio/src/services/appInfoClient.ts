// Asks the app for its name, version and identity
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { invoke } from '@tauri-apps/api/core';
import type { AppInfo } from '@liminal-hq/folio-protocol/generated/AppInfo';

/** What the page needs to know about the app it is running in. */
export interface AppInfoClient {
	appInfo(): Promise<AppInfo>;
}

/** The Rust `app_info` command, which the app itself registers (it is not a plugin command). */
export const tauriAppInfoClient: AppInfoClient = {
	appInfo: () => invoke<AppInfo>('app_info'),
};
