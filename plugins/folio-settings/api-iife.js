if ('__TAURI__' in window) {
var __TAURI_PLUGIN_FOLIO_SETTINGS__ = (function (exports, core, event) {
    'use strict';

    // Exposes typed guest-side wrappers for the folio-settings plugin
    //
    // (c) Copyright 2026 Liminal HQ, Scott Morris
    // SPDX-License-Identifier: Apache-2.0 OR MIT
    const PREFIX = 'plugin:folio-settings|';
    /** Sent to every window after every change; the payload is a `SettingsSnapshot`. */
    const SETTINGS_EVENT = 'folio-settings://changed';
    function cmd(name, args) {
        return core.invoke(`${PREFIX}${name}`, args);
    }
    /** Reports whether the settings plugin works. */
    function getStatus() {
        return cmd('get_status');
    }
    /** The settings in force and their revision. */
    function getSettings() {
        return cmd('get_settings');
    }
    /**
     * Changes the settings named in `patch` on top of what is in force now (Rust merges them), so a
     * window with an older copy of the document cannot turn another window's change back. Rejects with
     * a `SettingsCommandError` and changes nothing for a bad value or a save that fails.
     */
    function updateSettings(patch) {
        return cmd('update_settings', { patch });
    }
    /** Puts every setting back to its default. */
    function resetSettings() {
        return cmd('reset_settings');
    }
    /** Hears every change. Read `getSettings` first and apply snapshots with a higher revision. */
    function onSettingsChanged(listener) {
        return event.listen(SETTINGS_EVENT, (e) => listener(e.payload));
    }

    exports.SETTINGS_EVENT = SETTINGS_EVENT;
    exports.getSettings = getSettings;
    exports.getStatus = getStatus;
    exports.onSettingsChanged = onSettingsChanged;
    exports.resetSettings = resetSettings;
    exports.updateSettings = updateSettings;

    return exports;

})({}, __TAURI__.core, __TAURI__.event);
Object.defineProperty(window.__TAURI__, 'folioSettings', { value: __TAURI_PLUGIN_FOLIO_SETTINGS__ }) }
