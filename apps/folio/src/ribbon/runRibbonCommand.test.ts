// Tests for turning ribbon commands into menu commands and settings changes
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it, vi } from 'vitest';
import { runRibbonCommand } from './runRibbonCommand';

function setup() {
	return { run: vi.fn(async () => {}), update: vi.fn(async () => {}) };
}

describe('runRibbonCommand', () => {
	it('changes the mode', async () => {
		const { run, update } = setup();
		await runRibbonCommand({ id: 'mode', value: 'expert' }, run, update);
		expect(update).toHaveBeenCalledWith({ mode: 'expert' });
		expect(run).not.toHaveBeenCalled();
	});

	it('ignores a mode that is neither Easy nor Expert', async () => {
		const { run, update } = setup();
		await runRibbonCommand({ id: 'mode', value: 'wizard' }, run, update);
		expect(update).not.toHaveBeenCalled();
	});

	it('runs a File menu choice as a menu command', async () => {
		const { run, update } = setup();
		await runRibbonCommand({ id: 'file', value: 'file.saveAs' }, run, update);
		expect(run).toHaveBeenCalledWith('file.saveAs');
	});

	it('runs the menu command a control stands for', async () => {
		const { run, update } = setup();
		await runRibbonCommand({ id: 'undo' }, run, update);
		expect(run).toHaveBeenCalledWith('edit.undo');
		await runRibbonCommand({ id: 'ribbonToggle' }, run, update);
		expect(run).toHaveBeenLastCalledWith('view.ribbon');
	});

	it('does nothing for a tab change or a control with no command yet', async () => {
		const { run, update } = setup();
		await runRibbonCommand({ id: 'tab', value: 'insert' }, run, update);
		await runRibbonCommand({ id: 'bold', value: true }, run, update);
		expect(run).not.toHaveBeenCalled();
		expect(update).not.toHaveBeenCalled();
	});
});
