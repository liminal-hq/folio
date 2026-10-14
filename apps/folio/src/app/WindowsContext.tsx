// Shares the windows client with the menus that open windows
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { createContext, useContext } from 'react';
import { tauriWindowsClient, type WindowsClient } from '../services/windowsClient';

export const WindowsContext = createContext<WindowsClient>(tauriWindowsClient);

export function useWindows(): WindowsClient {
	return useContext(WindowsContext);
}
