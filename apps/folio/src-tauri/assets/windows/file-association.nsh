; Give Folio documents their own Explorer icon after Tauri registers the extension
;
; (c) Copyright 2026 Liminal HQ, Scott Morris
; SPDX-License-Identifier: Apache-2.0 OR MIT

!macro NSIS_HOOK_POSTINSTALL
  ; SHCTX follows Tauri's current-user or per-machine installation mode.
  WriteRegStr SHCTX "Software\Classes\Folio.folio\DefaultIcon" "" '$\"$INSTDIR\file-icons\folio-document.ico$\",0'
  !insertmacro UPDATEFILEASSOC
!macroend
