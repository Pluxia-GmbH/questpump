; The NSIS installer's hooks (tauri.conf.json bundle.windows.nsis.installerHooks). The installer runs per user and
; unelevated, so the helper made its firewall rule through its own one-time elevated step and left a marker in its data
; folder (runtime/firewall.rs). Uninstalling removes that rule with one UAC prompt, after the files are gone (a
; cancelled uninstall keeps the helper and its rule), and only a real uninstall: an update (/UPDATE, the in-app updater)
; and the uninstall a newer installer runs from its reinstall page (in place, so $EXEDIR is $INSTDIR; an uninstall from
; Windows' settings runs from a temporary copy) keep the rule, since the new helper has the same path. The marker stays:
; a declined prompt leaves the rule, and the next uninstall asks again. Tauri's uninstaller already removes the helper's
; start-at-login value (HKCU Run "QuestPump", the productName). The program and the player's data share
; %LOCALAPPDATA%\QuestPump (the per-user install folder is named after the productName, the data folder is the
; helper's own): the uninstaller removes only the files it installed (the program, the uninstaller and the bundled
; resources under resources\, which no data path equals), so the data stays and a reinstall keeps the progress. A real
; uninstall also removes the files the helper wrote into the games' folders (docs/plan.md slice 35: the Game State
; Integration cfg files, Rocket League's TAStatsAPI.ini), which it lists in its data folder's game-files
; (games/files.rs: UTF-16LE with a BOM, one path a line); a file the player's account may not delete stays, and an
; update keeps them all, since the new helper goes on using them.

!macro NSIS_HOOK_POSTUNINSTALL
	${If} $UpdateMode <> 1
	${AndIf} $EXEDIR != $INSTDIR
		${If} ${FileExists} "$LOCALAPPDATA\QuestPump\firewall-rule"
			ExecShellWait "runas" "$SYSDIR\netsh.exe" \
				'advfirewall firewall delete rule name=all dir=in program="$INSTDIR\${MAINBINARYNAME}.exe"' SW_HIDE
		${EndIf}
		Push $0
		Push $1
		Push $2
		ClearErrors
		FileOpen $0 "$LOCALAPPDATA\QuestPump\game-files" r
		${IfNot} ${Errors}
			; the BOM, when the file starts with one
			FileReadWord $0 $1
			${If} $1 <> 0xFEFF
				FileSeek $0 0 SET
			${EndIf}
			${Do}
				ClearErrors
				FileReadUTF16LE $0 $1
				${If} ${Errors}
					${ExitDo}
				${EndIf}
				; the line's CRLF
				StrCpy $2 $1 1 -1
				${If} $2 == "$\n"
					StrCpy $1 $1 -1
				${EndIf}
				StrCpy $2 $1 1 -1
				${If} $2 == "$\r"
					StrCpy $1 $1 -1
				${EndIf}
				${If} $1 != ""
					Delete $1
				${EndIf}
			${Loop}
			FileClose $0
		${EndIf}
		Pop $2
		Pop $1
		Pop $0
	${EndIf}
!macroend
