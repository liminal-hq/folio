// The message catalogue: every user-visible string, looked up by key
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** English (Canadian) messages. Add a key here, never a literal in a component. */
export const messages = {
	'app.name': 'Folio',
	'window.main.title': 'Folio',
	'main.loading': 'Starting Folio…',
	'main.running': 'Folio {version} is running on {platform}.',
	'main.identifier': 'Application identifier: {identifier}',
	'main.unavailable': 'The application details are not available here.',

	// The app menu and the menu bar (D-06)
	'menu.file': 'File',
	'menu.edit': 'Edit',
	'menu.view': 'View',
	'menu.help': 'Help',
	'menu.bar': 'Menu bar',
	'menu.more': 'More',
	'menu.unavailable': 'Not available yet',
	'command.new': 'New book…',
	'command.open': 'Open…',
	'command.openRecent': 'Open recent',
	'command.noRecent': 'No recent books',
	'command.importText': 'Import text…',
	'command.importLayout': 'Import layout…',
	'command.save': 'Save',
	'command.saveAs': 'Save as…',
	'command.export': 'Export book…',
	'command.showMenuBar': 'Show menu bar',
	'command.useRibbon': 'Use the ribbon',
	'command.expertMode': 'Expert mode',
	'command.jax': 'Jax',
	'command.preferences': 'Preferences…',
	'command.shortcuts': 'Keyboard shortcuts',
	'command.about': 'About Folio',
	'command.quit': 'Quit',
	'command.undo': 'Undo',
	'command.redo': 'Redo',
	'command.zoomIn': 'Zoom in',
	'command.zoomOut': 'Zoom out',

	// The main window's regions
	'region.toolbar': 'Toolbar',
	'region.toolbar.ribbon': 'The ribbon will go here.',
	'region.toolbar.classic': 'The classic toolbar will go here.',
	'region.stage': 'Page',
	'region.stage.empty': 'Open or start a book to lay out its pages.',
	'region.inspector': 'Inspector',
	'region.inspector.empty': 'Select something on the page to change it.',
	'region.shelf': 'Page shelf',
	'region.shelf.empty': 'The pages of your book will sit here.',
	'status.bar': 'Status',
	'status.noBook': 'No book open',
	'status.saved': 'Everything is saved',
	'status.jax': 'Jax',
	'mode.label': 'How do you like to work?',
	'mode.easy': 'Easy',
	'mode.expert': 'Expert',
} as const;

export type MessageKey = keyof typeof messages;

/** Looks up a message and fills its `{name}` placeholders. */
export function t(key: MessageKey, values: Record<string, string | number> = {}): string {
	return messages[key].replace(/\{(\w+)\}/g, (whole, name: string) =>
		name in values ? String(values[name]) : whole,
	);
}
