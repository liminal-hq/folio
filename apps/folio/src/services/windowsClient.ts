// Opens the app's other windows through the app's own `open_window` command
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { WindowKind } from '@liminal-hq/folio-protocol/generated/WindowKind';

/** Opens a window of a kind, or focuses it when it is already open. */
export interface WindowsClient {
	open(kind: WindowKind): Promise<void>;
	/**
	 * Opens the Book tools window on an area of tools, or on one tool in it (`tool` is a tool's id,
	 * such as `structure.contents`), or moves it there when it is open.
	 */
	openBookTools(area: string, tool?: string): Promise<void>;
}

/** The Rust `open_window` command, which the app itself registers (it is not a plugin command). */
export const tauriWindowsClient: WindowsClient = {
	open: (kind) => invoke<void>('open_window', { kind }),
	openBookTools: (area, tool) => invoke<void>('open_book_tools', { area, tool: tool ?? null }),
};

/** What the Book tools window was asked to open on: an area, and one tool in it when one was named. */
export interface ToolsAsked {
	area: string;
	tool: string | null;
}

/** What the Book tools window asks of the app: where it was told to open, and when that changes. */
export interface ToolsClient {
	area(): Promise<ToolsAsked>;
	onArea(listener: (asked: ToolsAsked) => void): Promise<() => void>;
}

export const tauriToolsClient: ToolsClient = {
	area: () => invoke<ToolsAsked>('book_tools_area'),
	onArea: (listener) => listen<ToolsAsked>('book-tools://area', (e) => listener(e.payload)),
};
