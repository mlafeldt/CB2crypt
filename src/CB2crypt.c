/*
 * CB2crypt.c -- Main project file
 *
 * Copyright (C) 2006 misfire
 * Copyright (C) 2003-2005 Parasyte
 * All rights reserved.
 *
 * This file is part of CB2crypt, the CodeBreaker PS2 Crypto Program.
 *
 * CB2crypt is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * CB2crypt is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 */

#include <windows.h>
#include "mytypes.h"
#include "mystring.h"
#include "resource.h"
#include "cb2_crypto.h"

// Application's name and current version
#define APP_NAME	"CB2crypt"
#define APP_VERSION	"v1.0"

// Title bar text for main window
#define TITLEBAR_TEXT	APP_NAME" "APP_VERSION

// Text for "About" message box
#define ABOUT_TEXT	"CodeBreaker PS2 Crypto Program "APP_VERSION"\n\n" \
			"Copyright (C) 2006 misfire\n\n" \
			"misfire@xploderfreax.de"

// Max text size for edit boxes
#define TEXTSIZE	(32*1024) // 32k

// Number of digits per cheat code
#define NUM_DIGITS_CODE	16

// Different modes for ParseText()
enum {
	MODE_DECRYPT,
	MODE_ENCRYPT,
	MODE_REFORMAT
};

// Win32 stuff
HWND hWindow;
HICON iWindow;
HFONT hFont, hNewFont;
HMENU hMenu;

BOOL CALLBACK MainDlg(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam);

int IsCheatCode(const char *s)
{
	int i = 0;

	while (*s) {
		if (isxdigit(*s)) i++;
		else if (!isspace(*s)) return FALSE;
		s++;
	}

	return (i == NUM_DIGITS_CODE);
}

void GetCheatCode(const char *s, u32 *code)
{
	char digits[NUM_DIGITS_CODE];
	int i = 0;

	while (*s) {
		if (isxdigit(*s)) digits[i++] = *s;
		s++;
	}

	sscanf(digits, "%08X%08X", &code[0], &code[1]);
}

/*
 * Parses the input text for codes. tbc
 */
int ParseText(int mode)
{
	char textin[TEXTSIZE+3]; // incl. CR, LF and NUL
	char textout[TEXTSIZE+3]; // should be sufficient
	char codestr[NUM_DIGITS_CODE+2]; // incl. space and NUL
	char *s;
	int count, linelen;
	u32 code[2];

	CBReset();
	memset(textout, 0, sizeof(textout));
	SetDlgItemText(hWindow, IDM_EDIT_OUT, "");

	// Get text
	count = GetDlgItemText(hWindow, IDM_EDIT_IN, textin, TEXTSIZE+1);
	if (!count) {
		MessageBox(hWindow, "You havn't entered any codes.",
			APP_NAME, MB_OK | MB_ICONINFORMATION | MB_APPLMODAL);
		return -1;
	}

	// Append CR, LF and NUL
	strncpy(&textin[count], "\r\n\0", 3);

	// Parse text
	s = textin;
	while (*s) {
		// Process line by line
		linelen = IndexOfChr(s, '\r');
		if (!IsEmptySubStr(s, linelen)) {
			s[linelen] = NUL;
			TrimStr(s);
			if (IsCheatCode(s)) {
				GetCheatCode(s, code);
				switch (mode) {
					case MODE_DECRYPT:
						CBDecryptCode(&code[0], &code[1]);
						break;
					case MODE_ENCRYPT:
						CBEncryptCode(&code[0], &code[1]);
						break;
					case MODE_REFORMAT:
						// Do nothing
						break;
				}
				sprintf(codestr, "%08X %08X", code[0], code[1]);
				strcat(textout, codestr);
			} else strcat(textout, s);
			strcat(textout, "\r\n");
		}
		// Next line
		s += linelen + 2;
	}

	// Display result
	SetDlgItemText(hWindow, IDM_EDIT_OUT, textout);

	return 0;
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow)
{
	MSG msg;
	RECT wndRect, dskRect;
	int w, h;

	// Create main dialog
	hWindow = CreateDialog(hInstance, IDD_MAIN_DLG, NULL, MainDlg);
	if (hWindow == NULL) {
		MessageBox(NULL, "Could not create main dialog", APP_NAME, MB_ICONERROR | MB_OK);
		return 0;
	}

	// Load and set dialog icons
	iWindow = LoadIcon(hInstance, IDI_LOCK);
	SendMessage(hWindow, WM_SETICON, ICON_BIG, (LPARAM)iWindow);
	SendMessage(hWindow, WM_SETICON, ICON_SMALL, (LPARAM)iWindow);

	// Change title bar to include version number
	SetWindowText(hWindow, TITLEBAR_TEXT);

	// Add system menu "About"
	hMenu = GetSystemMenu(hWindow, FALSE);
	AppendMenu(hMenu, MF_SEPARATOR, 0, NULL);
	AppendMenu(hMenu, MF_STRING, IDM_SYS_ABOUT, "About...");

	// Center dialog on screen
	GetWindowRect(hWindow, &wndRect);
	GetWindowRect(GetDesktopWindow(), &dskRect);
	w = wndRect.right - wndRect.left;
	h = wndRect.bottom - wndRect.top;
	MoveWindow(hWindow, (dskRect.right/2)-(w/2), (dskRect.bottom/2)-(h/2), w, h, TRUE);

	// Display dialog
	ShowWindow(hWindow, nCmdShow);
	UpdateWindow(hWindow);

	// Message loop
	while (GetMessage(&msg, NULL, 0, 0)) {
		if ((hWindow == NULL) || !IsDialogMessage(hWindow, &msg)) {
			TranslateMessage(&msg);
			DispatchMessage(&msg);
		}
	}

	return msg.wParam;
}

BOOL CALLBACK MainDlg(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
{
	LOGFONT lf;

	switch (uMsg) {
		case WM_INITDIALOG:
			// Setup font
			hFont = (HFONT)SendMessage(hwndDlg, WM_GETFONT, 0, 0);
			GetObject(hFont, sizeof(LOGFONT), &lf);
			strcpy(lf.lfFaceName, "Courier");
			hNewFont = CreateFontIndirect(&lf);
			SendDlgItemMessage(hwndDlg, IDM_EDIT_IN, WM_SETFONT, (WPARAM)hNewFont, FALSE);
			SendDlgItemMessage(hwndDlg, IDM_EDIT_OUT, WM_SETFONT, (WPARAM)hNewFont, FALSE);

			// Set text limits
			SendDlgItemMessage(hwndDlg, IDM_EDIT_IN, EM_SETLIMITTEXT, TEXTSIZE, 0);
			break;

		case WM_COMMAND:
			switch (HIWORD(wParam)) {
				case BN_CLICKED:
					switch (LOWORD(wParam)) {
						case IDM_DECRYPT:
							ParseText(MODE_DECRYPT);
							break;
						case IDM_ENCRYPT:
							ParseText(MODE_ENCRYPT);
							break;
						case IDM_REFORMAT:
							ParseText(MODE_REFORMAT);
							break;
						case IDM_CLOSE:
							PostQuitMessage(0);
							break;
						case IDM_CLEAR_IN:
							SetDlgItemText(hwndDlg, IDM_EDIT_IN, "");
							break;
						case IDM_CLEAR_OUT:
							SetDlgItemText(hwndDlg, IDM_EDIT_OUT, "");
							break;
					}
					break;
			}
			break;

		case WM_SYSCOMMAND:
			switch (LOWORD(wParam)) {
				case IDM_SYS_ABOUT:
					// Show "About" message box
					MessageBox(hWindow, ABOUT_TEXT, "About",
						MB_OK | MB_ICONINFORMATION | MB_APPLMODAL);
					break;
			}
			break;

		case WM_CLOSE:
		case WM_QUIT:
			PostQuitMessage(0);
			break;
	}

	return FALSE;
}
