// A dialog's frame, which is the page's dialog in the page and the whole window in a dialog window
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog, type DialogProps } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogWindowFrame } from '@liminal-hq/folio-chrome/DialogWindow/DialogWindowFrame';
import { createContext, useContext, type ReactNode } from 'react';
import { AppTitleBar } from '../app/AppTitleBar';

/** Set by a dialog window's host: the dialogs below it fill the window instead of floating. */
const DialogWindowContext = createContext<{ onDefaultAction?: () => void } | null>(null);

/** Something a dialog window wants said above its form, such as that the book changed under it. */
export const DialogNoticeContext = createContext<ReactNode>(null);

export function DialogWindowProvider({ children }: { children: ReactNode }) {
	return <DialogWindowContext.Provider value={{}}>{children}</DialogWindowContext.Provider>;
}

export interface DialogSurfaceProps extends DialogProps {
	/** In a dialog window: what Enter does in a field. */
	onDefaultAction?: () => void;
}

/**
 * The frame a dialog form draws itself in. In the page it is the shared `Dialog`; in a dialog window
 * it is the window's own content (the title bar is the window's), so one form serves both.
 */
export function DialogSurface({ onDefaultAction, ...props }: DialogSurfaceProps) {
	const inWindow = useContext(DialogWindowContext) !== null;
	const notice = useContext(DialogNoticeContext);
	if (!inWindow) return <Dialog {...props} />;
	if (!props.open) return null;
	const title = typeof props.title === 'string' ? props.title : '';
	return (
		<DialogWindowFrame
			title={title}
			titleBar={<AppTitleBar title={title} closeOnly />}
			description={props.description}
			notice={notice}
			footer={props.footer}
			onClose={props.onClose}
			onDefaultAction={onDefaultAction}
			initialFocus={props.initialFocus}
			dismissible={props.dismissible}
		>
			{props.children}
		</DialogWindowFrame>
	);
}
