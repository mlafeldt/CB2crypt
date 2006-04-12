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

 /* Includes */

#include <windows.h>
#include <stdio.h>
#include "mytypes.h"
#include "mystring.h"
#include "resource.h"
#include "cb2_crypto.h"

/* Defines */

// Set debug mode
//#define DEBUG

// Application's name and current version
#define APP_NAME	"CB2crypt"
#define APP_VERSION	"1.1"

// Title bar text for main window
#define TITLEBAR_TEXT	APP_NAME" v"APP_VERSION

// Text for "About" message box
#define ABOUT_TEXT	"CodeBreaker PS2 Crypto Program v"APP_VERSION"\n\n" \
			"Copyright (C) 2006 misfire\n\n" \
			"misfire@xploderfreax.de"

// Max text size for edit boxes
#define TEXTSIZE	(32*1024) // 32k

// Code parser defines
#define NUM_DIGITS_OCTET	8
#define NUM_DIGITS_CODE		(NUM_DIGITS_OCTET*2)

#define TOK_DELIMITER		" \t" // Space and tab
#define TOK_STEP_NUM		100

typedef struct {
	char	*str;
	int	type;
} token_t;

// Token types
enum {
	TOK_STRING,
	TOK_HEXOCTET,
	TOK_CODE,
	TOK_CODEADDR,
	TOK_CODEVAL
};

// Different modes for ParseText()
enum {
	MODE_DECRYPT,
	MODE_ENCRYPT,
	MODE_REFORMAT
};

/* Global variables */

// Flag: Use common V7 encryption?
int v7common = 0;

// Win32 stuff
HWND hWindow;
HICON iWindow;
HFONT hFont, hNewFont;
HMENU hMenu;

/* Local function prototypes */

BOOL CALLBACK MainDlg(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam);

/* Function definitions */

int GetTokenType(const char *s)
{
	int slen;

	if (IsHexStr(s)) {
		slen = strlen(s);
		if (slen == NUM_DIGITS_OCTET)
			return TOK_HEXOCTET;
		else if (slen == NUM_DIGITS_CODE)
			return TOK_CODE;
	}

	return TOK_STRING;
}

void GetCheatCode(const char *paddr, const char *pval, u32 *code)
{
	sscanf(paddr, "%08X", &code[0]);
	sscanf(pval,  "%08X", &code[1]);
}

/*
 * Parses the input text for codes.
 */
int ParseText(int mode)
{
	token_t *tok;
	int toknum, tokmax = TOK_STEP_NUM;
	char *p, *s;
	char textin[TEXTSIZE+3]; // incl. CR, LF and NUL
	char textout[TEXTSIZE+1024]; // 1k reserve, should be sufficient
	char codestr[NUM_DIGITS_CODE+2]; // incl. space and NUL
	u32 code[2];
	int i, ctrl, linelen;
#ifdef DEBUG
	FILE *fp;
#endif
	if (v7common)
		CBSetCommonV7();
	else
		CBReset();

	// Init
	tok = (token_t*)malloc(tokmax * sizeof(token_t));
	if (tok == NULL) return -1;
	memset(textin, 0, sizeof(textin));
	memset(textout, 0, sizeof(textout));
	memset(codestr, 0, sizeof(codestr));
	SetDlgItemText(hWindow, IDM_EDIT_OUT, "");

	// Get text from dialog box
	//i = GetDlgItemText(hWindow, IDM_EDIT_IN, textin, TEXTSIZE); // Weird results under Win98?
	GetDlgItemText(hWindow, IDM_EDIT_IN, textin, TEXTSIZE+1);
	i = strlen(textin);
	if (!i) {
		MessageBox(hWindow, "You havn't entered any codes.",
			APP_NAME, MB_OK | MB_ICONINFORMATION | MB_APPLMODAL);
		return -2;
	}

	// Append CR, LF and NUL
	textin[i] = NUL;
	strcat(textin, NEWLINE);

#ifdef DEBUG
	fp = fopen("dbg-in.bin", "wb");
	fwrite(textin, sizeof(textin), 1, fp);
	fclose(fp);
#endif
	// Parse text
	s = textin;
	while (*s) {
		// Process line by line
		linelen = IndexOfChr(s, '\r');
		if (!IsEmptySubStr(s, linelen)) {
			s[linelen] = NUL;

			// Get all tokens
			ctrl = 0;
			toknum = 0;
			p = strtok(s, TOK_DELIMITER);

			while (p != NULL) {
				tok[toknum].str = p;
				tok[toknum].type = GetTokenType(p);

				// Handle cheat code
				if (tok[toknum].type == TOK_HEXOCTET) {
					if (ctrl) {
						tok[toknum-1].type = TOK_CODEADDR;
						tok[toknum].type = TOK_CODEVAL;
						ctrl = 0;
					} else ctrl = 1;
				} else ctrl = 0;

				// Allocate more tokens if necessary
				if (++toknum == tokmax) {
					tokmax += TOK_STEP_NUM;
					tok = (token_t*)realloc(tok, tokmax * sizeof(token_t));
					if (tok == NULL) return -3;
				}

				// Next token
				p = strtok(NULL, TOK_DELIMITER);
			}

			// Process tokens, reassemble text
			ctrl = 0;
			i = 0;
			while (i < toknum) {
				if ((tok[i].type == TOK_CODE) || (tok[i].type == TOK_CODEADDR)) {
					if (tok[i].type == TOK_CODE) {
						GetCheatCode(tok[i].str, tok[i].str + NUM_DIGITS_OCTET, code);
					} else {
						GetCheatCode(tok[i].str, tok[i+1].str, code);
						i++;
					}

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

					if (i > 1) strcat(textout, NEWLINE);
					strcat(textout, codestr);
					ctrl = 1;
				} else {
					if (ctrl) {
						strcat(textout, NEWLINE);
						ctrl = 0;
					} else if (i > 0) strcat(textout, " ");
					strcat(textout, tok[i].str);
				}

				//printf("%i: %s\n", tok[i].type, tok[i].str);
				i++;
			}

			strcat(textout, NEWLINE);
		}

		// Next line
		s += linelen + 2;
	}

#ifdef DEBUG
	fp = fopen("dbg-out.bin", "wb");
	fwrite(textout, sizeof(textout), 1, fp);
	fclose(fp);
#endif
	// Display result
	SetDlgItemText(hWindow, IDM_EDIT_OUT, textout);

	free(tok);

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
						case IDM_COMMON_V7:
							v7common ^= 1;
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
