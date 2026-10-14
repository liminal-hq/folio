# Read aloud to catch typos

**Area:** Proofing and review · **Prototype:** `Folio Proofing.dc.html` · **Status:** artboard → merge into Folio

## What it does
The computer reads the book aloud while highlighting each word.

## UI
Play bar with speed, voice, from-here; follows the word on the page.

## Model and engine
System TTS (speech-dispatcher on Linux, SAPI/OneCore on Windows) via tauri plugin; offline.

## Jax
Jax can be the reader voice — optional.

## Done when
- Works in Easy with plain words; Expert shows the numbers.
- Undo covers every change in one step per action.
- Preflight (05) knows about it where relevant.
