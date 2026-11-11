// Exposes typed guest-side wrappers for the window-dialogs plugin
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

const PREFIX = 'plugin:window-dialogs|';

/** Sent to the opener when a dialog is done: its result, or a cancellation. */
export const FINISHED_EVENT = 'window-dialogs://finished';
/** Sent to the opener when a modal dialog blocks it or lets it go. */
export const MODAL_EVENT = 'window-dialogs://modal';
/** Sent to the opener when a dialog posts a message to it. */
export const MESSAGE_EVENT = 'window-dialogs://message';

/** How a dialog window is made, in logical pixels. */
export interface DialogSpec {
	title: string;
	width: number;
	height: number;
	minWidth?: number;
	minHeight?: number;
	resizable?: boolean;
	/** Blocks the opener while the dialog is open. Defaults to true. */
	modal?: boolean;
	/** Frameless, transparent and shadowed. Defaults to true. */
	chromeless?: boolean;
	/** The page to load; defaults to `index.html`. */
	url?: string;
}

export interface Opened {
	requestId: number;
	/** True when the dialog was already open and was brought forward. */
	existing: boolean;
}

/** What a dialog window was opened for. */
export interface DialogRequest<P = unknown> {
	requestId: number;
	label: string;
	payload: P;
	modal: boolean;
}

/** How a dialog ended. `result` is null when it was cancelled. */
export interface Finished<T = unknown> {
	requestId: number;
	label: string;
	cancelled: boolean;
	result: T | null;
}

export interface ModalState {
	open: boolean;
	label: string;
	/** True when the platform blocks the opener itself; false when the page has to. */
	native: boolean;
}

export interface Posted<M = unknown> {
	requestId: number;
	label: string;
	message: M;
}

/** Opens the dialog window `label` for the calling window. Refused while it has another modal open. */
export function openDialog(label: string, spec: DialogSpec, payload: unknown): Promise<Opened> {
	return invoke<Opened>(`${PREFIX}open_dialog`, { label, spec, payload });
}

/** What the calling dialog window was opened for. Ask when the dialog starts. */
export function dialogRequest<P = unknown>(): Promise<DialogRequest<P>> {
	return invoke<DialogRequest<P>>(`${PREFIX}dialog_request`);
}

/** Says the dialog has painted; returns the milliseconds since it was opened. */
export function dialogShown(): Promise<number> {
	return invoke<number>(`${PREFIX}dialog_shown`);
}

/** Ends the calling dialog with a result: the opener hears it, and the window closes. */
export function finishDialog(result: unknown): Promise<void> {
	return invoke<void>(`${PREFIX}finish_dialog`, { result });
}

/** Sends a message from the calling dialog to its opener. */
export function postToOpener(message: unknown): Promise<void> {
	return invoke<void>(`${PREFIX}post_to_opener`, { message });
}

/** Brings the calling window's modal dialog forward. */
export function focusDialog(): Promise<void> {
	return invoke<void>(`${PREFIX}focus_dialog`);
}

/** Hears how the dialogs this window opened ended. */
export function onFinished<T = unknown>(
	listener: (finished: Finished<T>) => void,
): Promise<UnlistenFn> {
	return listen<Finished<T>>(FINISHED_EVENT, (event) => listener(event.payload));
}

/** Hears when a modal dialog blocks this window or lets it go. */
export function onModal(listener: (state: ModalState) => void): Promise<UnlistenFn> {
	return listen<ModalState>(MODAL_EVENT, (event) => listener(event.payload));
}

/** Hears messages the dialogs this window opened post to it. */
export function onMessage<M = unknown>(listener: (posted: Posted<M>) => void): Promise<UnlistenFn> {
	return listen<Posted<M>>(MESSAGE_EVENT, (event) => listener(event.payload));
}
