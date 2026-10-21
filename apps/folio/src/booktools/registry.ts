// Every Book tool, in the order each area's sidebar lists them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { ChecklistTool } from './publishing/ChecklistTool';
import { KeywordsTool } from './publishing/KeywordsTool';
import { RoyaltyTool } from './publishing/RoyaltyTool';
import { PunctuationTool } from './typesetting/PunctuationTool';
import { StylesTool } from './typesetting/StylesTool';
import { AltTextTool } from './pictures/AltTextTool';
import { PicturesTool } from './pictures/PicturesTool';
import type { ToolArea, ToolDef } from './tools';

export const TOOLS: ToolDef[] = [
	{
		id: 'typesetting.punctuation',
		area: 'typesetting',
		titleKey: 'tools.punctuation.title',
		descKey: 'tools.punctuation.desc',
		Component: PunctuationTool,
	},
	{
		id: 'typesetting.styles',
		area: 'typesetting',
		titleKey: 'tools.styles.title',
		descKey: 'tools.styles.desc',
		Component: StylesTool,
	},
	{
		id: 'publishing.checklist',
		area: 'publishing',
		titleKey: 'tools.checklist.title',
		descKey: 'tools.checklist.desc',
		Component: ChecklistTool,
	},
	{
		id: 'publishing.keywords',
		area: 'publishing',
		titleKey: 'tools.keywords.title',
		descKey: 'tools.keywords.desc',
		Component: KeywordsTool,
	},
	{
		id: 'publishing.royalty',
		area: 'publishing',
		titleKey: 'tools.royalty.title',
		descKey: 'tools.royalty.desc',
		Component: RoyaltyTool,
	},
	{
		id: 'pictures.list',
		area: 'pictures',
		titleKey: 'tools.pictures.title',
		descKey: 'tools.pictures.desc',
		Component: PicturesTool,
	},
	{
		id: 'pictures.alt',
		area: 'pictures',
		titleKey: 'tools.alt.title',
		descKey: 'tools.alt.desc',
		Component: AltTextTool,
	},
];

export function toolsIn(area: ToolArea): ToolDef[] {
	return TOOLS.filter((tool) => tool.area === area);
}
