// The commands that open one Book tool, and the tool each opens
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** Each command and the id of the tool it opens, which starts with the tool's area. */
export const TOOL_COMMANDS = {
	'tool.copyright': 'structure.copyright',
	'tool.contents': 'structure.contents',
	'tool.punctuation': 'typesetting.punctuation',
	'tool.styles': 'typesetting.styles',
	'tool.pictures': 'pictures.list',
	'tool.altText': 'pictures.alt',
	'tool.checklist': 'publishing.checklist',
	'tool.keywords': 'publishing.keywords',
	'tool.royalty': 'publishing.royalty',
} as const;

export type ToolCommandId = keyof typeof TOOL_COMMANDS;
