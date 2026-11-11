if ('__TAURI__' in window) {
var __TAURI_PLUGIN_WINDOW_DIALOGS__ = (function (exports, core, event) {
    'use strict';

    // Exposes typed guest-side wrappers for the window-dialogs plugin
    //
    // (c) Copyright 2026 Liminal HQ, Scott Morris
    // SPDX-License-Identifier: Apache-2.0 OR MIT
    const PREFIX = 'plugin:window-dialogs|';
    /** Sent to the opener when a dialog is done: its result, or a cancellation. */
    const FINISHED_EVENT = 'window-dialogs://finished';
    /** Sent to the opener when a modal dialog blocks it or lets it go. */
    const MODAL_EVENT = 'window-dialogs://modal';
    /** Sent to the opener when a dialog posts a message to it. */
    const MESSAGE_EVENT = 'window-dialogs://message';
    /** Opens the dialog window `label` for the calling window. Refused while it has another modal open. */
    function openDialog(label, spec, payload) {
        return core.invoke(`${PREFIX}open_dialog`, { label, spec, payload });
    }
    /** What the calling dialog window was opened for. Ask when the dialog starts. */
    function dialogRequest() {
        return core.invoke(`${PREFIX}dialog_request`);
    }
    /** Says the dialog has painted; returns the milliseconds since it was opened. */
    function dialogShown() {
        return core.invoke(`${PREFIX}dialog_shown`);
    }
    /** Ends the calling dialog with a result: the opener hears it, and the window closes. */
    function finishDialog(result) {
        return core.invoke(`${PREFIX}finish_dialog`, { result });
    }
    /** Sends a message from the calling dialog to its opener. */
    function postToOpener(message) {
        return core.invoke(`${PREFIX}post_to_opener`, { message });
    }
    /** Brings the calling window's modal dialog forward. */
    function focusDialog() {
        return core.invoke(`${PREFIX}focus_dialog`);
    }
    /** Hears how the dialogs this window opened ended. */
    function onFinished(listener) {
        return event.listen(FINISHED_EVENT, (event) => listener(event.payload));
    }
    /** Hears when a modal dialog blocks this window or lets it go. */
    function onModal(listener) {
        return event.listen(MODAL_EVENT, (event) => listener(event.payload));
    }
    /** Hears messages the dialogs this window opened post to it. */
    function onMessage(listener) {
        return event.listen(MESSAGE_EVENT, (event) => listener(event.payload));
    }

    exports.FINISHED_EVENT = FINISHED_EVENT;
    exports.MESSAGE_EVENT = MESSAGE_EVENT;
    exports.MODAL_EVENT = MODAL_EVENT;
    exports.dialogRequest = dialogRequest;
    exports.dialogShown = dialogShown;
    exports.finishDialog = finishDialog;
    exports.focusDialog = focusDialog;
    exports.onFinished = onFinished;
    exports.onMessage = onMessage;
    exports.onModal = onModal;
    exports.openDialog = openDialog;
    exports.postToOpener = postToOpener;

    return exports;

})({}, __TAURI__.core, __TAURI__.event);
Object.defineProperty(window.__TAURI__, 'windowDialogs', { value: __TAURI_PLUGIN_WINDOW_DIALOGS__ }) }
