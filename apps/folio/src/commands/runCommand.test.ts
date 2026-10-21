// Tests for running menu commands
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { SettingsPatch } from '@liminal-hq/folio-plugin-settings';
import { describe, expect, it, vi } from 'vitest';
import { DEFAULT_SETTINGS } from '../settings/fakeSettingsClient';
import { availableCommands, runCommand, type CommandContext } from './runCommand';

const OPEN = availableCommands({ hasBook: true, canUndo: true, canRedo: true });
const NONE = availableCommands({ hasBook: false, canUndo: false, canRedo: false });

function context(overrides: Partial<CommandContext['settings']> = {}) {
	const patches: SettingsPatch[] = [];
	const books = {
		newBook: vi.fn(async () => {}),
		open: vi.fn(async () => {}),
		save: vi.fn(async () => {}),
		saveAs: vi.fn(async () => {}),
		close: vi.fn(async () => {}),
		undo: vi.fn(async () => {}),
		redo: vi.fn(async () => {}),
		exportBook: vi.fn(async () => {}),
		importText: vi.fn(async () => {}),
	};
	const closeWindow = vi.fn();
	const openWindow = vi.fn();
	const openBookTools = vi.fn();
	const value: CommandContext = {
		settings: { ...DEFAULT_SETTINGS, ...overrides },
		update: async (patch) => {
			patches.push(patch);
		},
		openWindow,
		openBookTools,
		closeWindow,
		books,
	};
	return { value, patches, closeWindow, openWindow, openBookTools, books };
}

describe('availableCommands', () => {
	it('keeps the book commands for when there is a book', () => {
		expect(NONE.has('file.new') && NONE.has('file.open')).toBe(true);
		expect(NONE.has('file.save') || NONE.has('file.close') || NONE.has('file.saveAs')).toBe(false);
		expect(OPEN.has('file.save') && OPEN.has('file.saveAs') && OPEN.has('file.close')).toBe(true);
		expect(OPEN.has('file.export') && !NONE.has('file.export')).toBe(true);
	});

	it('offers undo and redo only when there is something to undo or redo', () => {
		expect(
			availableCommands({ hasBook: true, canUndo: true, canRedo: false }).has('edit.redo'),
		).toBe(false);
		expect(
			availableCommands({ hasBook: true, canUndo: false, canRedo: true }).has('edit.undo'),
		).toBe(false);
	});
});

describe('runCommand', () => {
	it('toggles the menu bar, the toolbar, the mode and Jax', async () => {
		const bar = context();
		await runCommand('view.menuBar', bar.value, NONE);
		expect(bar.patches).toEqual([{ interface: { menuBar: true } }]);

		const ribbon = context();
		await runCommand('view.ribbon', ribbon.value, NONE);
		expect(ribbon.patches).toEqual([{ interface: { toolbar: 'classic' } }]);

		const easy = context();
		await runCommand('view.expert', easy.value, NONE);
		expect(easy.patches).toEqual([{ mode: 'expert' }]);
		const expert = context({ mode: 'expert' });
		await runCommand('view.expert', expert.value, NONE);
		expect(expert.patches).toEqual([{ mode: 'easy' }]);

		const on = context();
		await runCommand('view.jax', on.value, NONE);
		expect(on.patches).toEqual([{ jax: 'off' }]);
		const off = context({ jax: 'off' });
		await runCommand('view.jax', off.value, NONE);
		expect(off.patches).toEqual([{ jax: 'tips' }]);
	});

	it('opens Preferences and About, and quits by closing the window', async () => {
		const { value, openWindow, closeWindow } = context();
		await runCommand('app.preferences', value, NONE);
		await runCommand('app.about', value, NONE);
		expect(openWindow.mock.calls).toEqual([['Preferences'], ['About']]);
		await runCommand('app.quit', value, NONE);
		expect(closeWindow).toHaveBeenCalledOnce();
	});

	it('opens the Book tools window on the group of tools asked for', async () => {
		const { value, openBookTools } = context();
		await runCommand('tools.pictures', value, NONE);
		await runCommand('tools.structure', value, NONE);
		expect(openBookTools.mock.calls).toEqual([['pictures'], ['structure']]);
	});

	it('opens the New book wizard for File → New book…', async () => {
		const { value, openWindow } = context();
		await runCommand('file.new', value, NONE);
		expect(openWindow.mock.calls).toEqual([['WizardNew']]);
	});

	it('runs the book commands', async () => {
		const { value, books } = context();
		await runCommand('file.open', value, OPEN);
		await runCommand('file.save', value, OPEN);
		await runCommand('file.saveAs', value, OPEN);
		await runCommand('file.close', value, OPEN);
		await runCommand('file.export', value, OPEN);
		await runCommand('file.importText', value, OPEN);
		await runCommand('edit.undo', value, OPEN);
		await runCommand('edit.redo', value, OPEN);
		for (const call of Object.values(books)) expect(call).toHaveBeenCalledOnce();
	});

	it('opens a recent book by the path in its id, even with nothing open', async () => {
		const { value, books } = context();
		await runCommand('file.openRecent:/books/otter.folio', value, NONE);
		expect(books.open).toHaveBeenCalledWith('/books/otter.folio');
	});

	it('does nothing for a command that is not available or is unknown', async () => {
		const { value, books, patches } = context();
		await runCommand('file.save', value, NONE);
		await runCommand('edit.undo', value, NONE);
		await runCommand('file.export', value, NONE);
		await runCommand('nonsense', value, OPEN);
		expect(patches).toEqual([]);
		for (const call of Object.values(books)) expect(call).not.toHaveBeenCalled();
	});
});
