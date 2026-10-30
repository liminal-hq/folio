// An update client for tests: it answers from a state the test sets, and records what it was asked
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { UpdateState } from '@liminal-hq/folio-protocol/generated/UpdateState';
import type { UpdateClient } from './updateClient';

export interface FakeUpdateClient {
	client: UpdateClient;
	/** The calls made, such as `check()`. */
	calls: string[];
	/** Sets the state and tells whoever is listening, as the app would. */
	emit(state: UpdateState): void;
	/** What `check()` finds. */
	checkFinds: UpdateState;
	/** What `download()` ends in. */
	downloadEnds: UpdateState;
}

export function createFakeUpdateClient(initial: UpdateState = { kind: 'idle' }): FakeUpdateClient {
	const listeners = new Set<(state: UpdateState) => void>();
	let current = initial;
	const fake: FakeUpdateClient = {
		calls: [],
		checkFinds: { kind: 'upToDate' },
		downloadEnds: { kind: 'ready', version: '0.2.0' },
		emit(state) {
			current = state;
			for (const listener of listeners) listener(state);
		},
		client: {
			async state() {
				fake.calls.push('state()');
				return current;
			},
			async check() {
				fake.calls.push('check()');
				fake.emit({ kind: 'checking' });
				fake.emit(fake.checkFinds);
				return fake.checkFinds;
			},
			async download() {
				fake.calls.push('download()');
				fake.emit(fake.downloadEnds);
				return fake.downloadEnds;
			},
			async install() {
				fake.calls.push('install()');
			},
			async onState(listener) {
				listeners.add(listener);
				return () => listeners.delete(listener);
			},
		},
	};
	return fake;
}
