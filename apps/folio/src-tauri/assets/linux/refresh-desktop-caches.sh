#!/bin/sh
# Refresh file-type and icon caches after installing or removing Folio
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

set -eu

# Minimal systems may not have a desktop or its cache tools installed.
if command -v update-mime-database >/dev/null 2>&1 && [ -d /usr/share/mime ]; then
	update-mime-database /usr/share/mime || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
	update-desktop-database /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1 && [ -d /usr/share/icons/hicolor ]; then
	gtk-update-icon-cache -q -f -t /usr/share/icons/hicolor || true
fi
