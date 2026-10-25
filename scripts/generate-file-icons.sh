#!/usr/bin/env bash
# Render the Folio document icon for Linux file managers and Windows Explorer
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

command -v rsvg-convert >/dev/null || { echo 'Install librsvg to render the document icon.' >&2; exit 1; }
if command -v magick >/dev/null; then
	image_command=magick
elif command -v convert >/dev/null; then
	image_command=convert
else
	echo 'Install ImageMagick to create the Windows icon.' >&2
	exit 1
fi

icon_dir=apps/folio/src-tauri/assets/file-icons
icon_sizes=(16 24 32 48 64 128 256)
icon_frames=()
for size in "${icon_sizes[@]}"; do
	frame="${icon_dir}/${size}x${size}.png"
	rsvg-convert --width "$size" --height "$size" "${icon_dir}/folio-document.svg" --output "$frame"
	icon_frames+=("$frame")
done
"$image_command" "${icon_frames[@]}" "${icon_dir}/folio-document.ico"
