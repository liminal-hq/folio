// Asks the app to look for, fetch and install a newer Folio, and hears how that is going
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { UpdateState } from '@liminal-hq/folio-protocol/generated/UpdateState';

/** What the update chip and dialog need from the app. */
export interface UpdateClient {
	state(): Promise<UpdateState>;
	/** Looks for a newer version now. */
	check(): Promise<UpdateState>;
	/** Fetches the version found and checks its signature. */
	download(): Promise<UpdateState>;
	/** Installs what was fetched and restarts Folio; rejects with why not. */
	install(): Promise<void>;
	/** Hears each change of state; answers a function that stops listening. */
	onState(listener: (state: UpdateState) => void): Promise<() => void>;
}

/** The Rust `update_*` commands and the `update-state` event, which the app itself registers. */
export const tauriUpdateClient: UpdateClient = {
	state: () => invoke<UpdateState>('update_state'),
	check: () => invoke<UpdateState>('update_check'),
	download: () => invoke<UpdateState>('update_download'),
	install: () => invoke<void>('update_install'),
	onState: (listener) => listen<UpdateState>('update-state', (event) => listener(event.payload)),
};
