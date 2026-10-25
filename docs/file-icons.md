# Folio document icons

The application icon identifies Folio itself. The document icon identifies a `.folio` book: a warm paper sheet with a folded corner and Folio's book mark. Its editable source is `apps/folio/src-tauri/assets/file-icons/folio-document.svg`.

<img src="../apps/folio/src-tauri/assets/file-icons/128x128.png" alt="Folio book document icon" width="128" height="128">

Run `bun run icons:files` after editing the SVG. This uses `rsvg-convert` from librsvg and ImageMagick (`magick` or `convert`) to regenerate the committed 16, 24, 32, 48, 64, 128 and 256 px PNGs and the multiresolution Windows ICO. These tools are only needed to edit the artwork, not to build Folio.

## Linux

Tauri's `bundle.fileAssociations` and the desktop template advertise the package's existing MIME type, `application/vnd.liminal.folio+zip`. Debian and RPM bundles install the shared MIME-info definition under `/usr/share/mime/packages` and the SVG and PNG icons under the `hicolor` theme's `mimetypes` directories. The icon basename follows the MIME type with its slash changed to a hyphen. The MIME definition also accepts the earlier `application/x-folio` name as an alias and recognises the package's first, uncompressed `mimetype` ZIP entry.

Package installation and removal refresh the MIME, desktop-entry and icon caches when their tools are available. AppImages carry the same assets inside their AppDir; host desktop integration remains the responsibility of the AppImage integration tool or the person installing it.

## Windows

Tauri v2's file-association configuration has no document-icon field. The Windows-specific configuration bundles `file-icons/folio-document.ico` as an application resource. The NSIS post-install hook assigns it to `Folio.folio\DefaultIcon` in the installer-selected registry scope and refreshes Explorer's associations. Tauri owns the extension registration and its removal.

The MSI WiX fragment adds the same icon registry value under the machine-wide `Folio.folio` class used by Tauri's MSI association. Windows Installer owns that value and removes it with its component. The artwork is shared by x64 and ARM64 installers.

These changes register file metadata and artwork. They do not add a thumbnail provider or change how Folio handles a file path passed to the application.

## Verification

- Regenerate the artwork and inspect its 16 and 32 px frames as well as the SVG.
- Validate the MIME XML with `update-mime-database` in an isolated data directory, then query both a `.folio` filename and a ZIP whose first entry contains Folio's MIME type.
- Inspect built Debian and RPM package contents for the MIME definition and the `hicolor` assets.
- On Windows, install each installer, check the icon of a saved `.folio` file in Explorer, then uninstall and check that the association and icon resource are removed.

## References

- [Tauri file-association configuration](https://v2.tauri.app/reference/config/#fileassociation)
- [Tauri Windows installer extensions](https://v2.tauri.app/distribute/windows-installer/#extending-the-installer)
- [Shared MIME-info database specification](https://specifications.freedesktop.org/shared-mime-info/latest/ar01s02.html)
