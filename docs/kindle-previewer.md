# Checking an EPUB the way Amazon will

KDP converts an uploaded EPUB with the same engine as Kindle Previewer. A book that converts there without errors is one KDP will take, so Folio checks the EPUBs it makes against it.

## What goes to KDP

An ebook upload is two files. The manuscript is the EPUB. The cover is a separate JPEG or TIFF, 2,560 × 1,600 pixels at best, RGB, under 50 MB; Folio's _Cover picture for a store_ is that file. The EPUB may carry a cover as well, and Folio's fixed-layout EPUB does, as its first page. Kindle Previewer says `W14016: Cover not specified` of a reflowable EPUB with none, and calls it safe to ignore because the cover is uploaded separately.

## Running the check

```bash
scripts/kindle-previewer.sh setup               # once: download Previewer and unpack its command line
scripts/kindle-previewer.sh check BOOK.epub...  # convert each book and print Amazon's log
scripts/kindle-previewer.sh check --keep DIR BOOK.epub   # also keep the Kindle file and the logs
```

`check` exits 1 if any book fails to convert, so it can gate a script.

Previewer's app is a Windows and macOS program, but its package holds `KindlePreviewerCLI.exe`, which converts and validates with no window. On Linux `setup` unpacks it into `~/.cache/folio/kindle-previewer` (about 900 MB) and `check` runs it under Wine, which needs `wine`, `7z` and `curl`. On Windows or macOS install Kindle Previewer and set `KINDLE_PREVIEWER_CLI` to its `KindlePreviewerCLI` and the script uses that.

The converter writes its result only when its output is a pipe, and leaves a working folder behind that fails the next run of a book with the same name; the script handles both.

## What it has found

- A fixed-layout EPUB without `original-resolution` metadata fails with `E34002`. Folio writes it from the page size.

## Other checks

- **EPUBCheck** (W3C) checks the package against the EPUB standard. It needs Java and is not part of this script.
- **Kindle Previewer's window** shows the pages as a Kindle draws them. The command line does not.
- **KDP's online previewer**, after an upload, uses the same engine.
- A real device is the only check of how a page looks: Kobo was checked on an Aura, Kindle has not been.
