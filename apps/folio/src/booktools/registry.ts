// Every Book tool, in the order each area's sidebar lists them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { AltTextTool } from './pictures/AltTextTool';
import { PicturesTool } from './pictures/PicturesTool';
import type { ToolArea, ToolDef } from './tools';

export const TOOLS: ToolDef[] = [
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
