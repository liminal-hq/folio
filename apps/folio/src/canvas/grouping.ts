// When frames can be grouped and when a group can be taken apart
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { Surface } from '@liminal-hq/folio-protocol/generated/Surface';

/** A frame the writer can change: not locked, not a master's. */
const free = (frame: Frame) => !frame.locked && !frame.fromMaster;

/**
 * Whether the selected frames can be grouped: two or more, all free, on a page and not on a master
 * page, and none carrying a story from frame to frame (the book would refuse these).
 */
export function canGroup(frames: readonly Frame[], onMaster: boolean): boolean {
	return (
		!onMaster &&
		frames.length >= 2 &&
		frames.every((f) => free(f) && !(f.type === 'text' && f.flow))
	);
}

/** Whether the selection is one free group, which can be taken apart. */
export function canUngroup(frames: readonly Frame[]): boolean {
	return frames.length === 1 && frames[0]!.type === 'group' && free(frames[0]!);
}

/** The intent that groups the frames on `on`. */
export function groupIntent(frames: readonly Frame[], on: Surface): Intent {
	return { type: 'GroupFrames', on, frames: frames.map((f) => f.id) };
}

/** The intent that takes the group apart. */
export function ungroupIntent(group: Frame, on: Surface): Intent {
	return { type: 'UngroupFrames', on, frame: group.id };
}

/** The frames that were in a group, the ids to select after taking it apart. */
export function childIds(group: Frame): string[] {
	return group.type === 'group' ? group.children.map((c) => c.id) : [];
}
