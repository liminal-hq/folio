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
	'command.bookTools': 'Book tools',
	'command.preferences': 'Preferences…',
	'command.shortcuts': 'Keyboard shortcuts',
	'command.about': 'About Folio',
	'command.quit': 'Quit',
	'command.undo': 'Undo',
	'command.redo': 'Redo',
	'command.zoomIn': 'Zoom in',
	'command.zoomOut': 'Zoom out',

	// Window titles
	'window.about.title': 'About Folio',
	'window.preferences.title': 'Preferences',
	'window.bookTools.title': 'Book Tools',

	// About
	'about.tagline': 'Books, laid out by hand, without the hard parts.',
	'about.version': 'Version {version}',
	'about.identifier': '{identifier} on {platform}',
	'about.licence': 'Licensed under either the Apache License 2.0 or the MIT licence.',
	'about.copyright': 'Copyright 2026 Liminal HQ, Scott Morris',
	'about.copyDetails': 'Copy build details',
	'about.copied': 'Copied',
	'about.details': 'Folio {version} ({identifier}) on {platform}',

	// Book tools
	'bookTools.heading': 'Book tools',
	'bookTools.empty':
		'The workbenches for structure, typesetting, pictures and publishing will open here.',

	// Preferences
	'preferences.section.interface': 'Interface',
	'preferences.section.appearance': 'Appearance',
	'preferences.section.units': 'Units',
	'preferences.section.help': 'Help',
	'preferences.menuBar': 'Show the menu bar',
	'preferences.menuBar.description': 'Alt or F10 still opens it while it is hidden.',
	'preferences.toolbar': 'Toolbar',
	'preferences.toolbar.ribbon': 'Ribbon',
	'preferences.toolbar.classic': 'Classic',
	'preferences.largerControls': 'Larger controls',
	'preferences.largerControls.description': 'Make buttons and rows easier to hit.',
	'preferences.mode': 'How do you like to work?',
	'preferences.mode.description': 'Easy hides what you do not need. Expert shows everything.',
	'preferences.colourMode': 'Colour scheme',
	'preferences.colourMode.system': 'Follow system',
	'preferences.colourMode.light': 'Light',
	'preferences.colourMode.dark': 'Dark',
	'preferences.units': 'Show lengths in',
	'preferences.units.description': 'Books store lengths in points. This changes only what you see.',
	'preferences.units.inches': 'Inches',
	'preferences.units.millimetres': 'Millimetres',
	'preferences.jax': 'Jax',
	'preferences.jax.description': 'An optional helper. He never interrupts.',
	'preferences.jax.guide': 'Walk me through it',
	'preferences.jax.tips': 'Just tips',
	'preferences.jax.off': 'No thanks',
	'preferences.error': 'That change could not be made: {message}',

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
