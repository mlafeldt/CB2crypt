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
//#define _DEBUG

// Application's name and current version
#define APP_NAME		"CB2crypt"
#define APP_VERSION		"1.21"

// Title bar text for main window
#define TITLEBAR_TEXT		APP_NAME" v"APP_VERSION

// E-mail address for About box
#define EMAIL_ADDR		"mailto:misfire@xploderfreax.de?subject="APP_NAME

// Max text size for edit boxes
// FIXME: Win98/ME can only handle ~30k
#define TEXTSIZE		(32*1024) // 32k

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
#define TOK_WILDCARD		(1 << 0)
#define TOK_STRING		(1 << 1)
#define TOK_HEXOCTET		(1 << 2)
#define TOK_CODE		(1 << 3) // Address and value, w/o space
#define TOK_CODEADDR		(1 << 4)
#define TOK_CODEVAL		(1 << 5)

// Different modes for ParseText()
enum {
	MODE_DECRYPT,
	MODE_ENCRYPT,
	MODE_REFORMAT
};

// Macro to convert a hex string into u32
#define STR_TO_U32(s, u)	sscanf(s, "%08X", u)

/* Global variables */

// Flag: Use common V7 encryption?
int v7common = 0;

/* Local function prototypes */

int MsgBox(HWND hwnd, UINT uType, const char *szFormat, ...);
LRESULT CALLBACK WndProc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam);
BOOL CALLBACK MainDlgProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam);
BOOL CALLBACK AboutDlgProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam);

/* Function definitions */

/*
 * Returns the type of a string (token).
 */
int GetTokenType(const char *s)
{
	int i, len = strlen(s);

	if ((len == NUM_DIGITS_OCTET) || (len == NUM_DIGITS_CODE)) {
		if (IsHexStr(s))
			return (len == NUM_DIGITS_OCTET) ? TOK_HEXOCTET : TOK_CODE;

		// Wildcard?
		for (i = 0; i < len; i++) {
			// "?" is only allowed in code value
			if ((len == NUM_DIGITS_OCTET) || (i >= NUM_DIGITS_OCTET)) {
				if (!isxdigit(s[i]) && (s[i] != '?'))
					return TOK_STRING;
			} else if (!isxdigit(s[i]))
					return TOK_STRING;
		}

		return ((len == NUM_DIGITS_OCTET) ? TOK_HEXOCTET : TOK_CODE) | TOK_WILDCARD;
	}

	return TOK_STRING;
}

/*
 * Computes the CRC-16 value of a data buffer.
 */
#ifdef _DEBUG
u16 GetCrc16(const u8 *blk, int len)
{
	static const u16 tbl[] = {
		0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401,
		0xA001, 0x6C00, 0x7800, 0xB401, 0x5000, 0x9C01, 0x8801, 0x4400
	};

	u16 crc = 0;
	int i;

	for (i = 0; i < len; i++) {
		crc = (crc >> 4) ^ tbl[(blk[i] ^ crc) & 0xF];
		crc = (crc >> 4) ^ tbl[((blk[i] >> 4) ^ crc) & 0xF];
	}

	return crc;
}
#endif

/*
 * Parses the text from the input dialog box for codes, processes them and
 * displays the result in the output dialog box.
 */
int ParseText(HWND hwnd, int mode)
{
	token_t *tok;
	int toknum, tokmax = TOK_STEP_NUM;
	char *p, *s;
	char textin[TEXTSIZE+3]; // incl. CR, LF and NUL
	char textout[TEXTSIZE+1024]; // 1k reserve, should be sufficient
	char codestr[NUM_DIGITS_CODE+2]; // incl. space and NUL
	u32 code[2];
	int i, ctrl, linelen;
#ifdef _DEBUG
	FILE *fp = fopen("tokens.dbg", "w");
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
	SetDlgItemText(hwnd, IDM_EDIT_OUT, "");

	// Get text from dialog box
	GetDlgItemText(hwnd, IDM_EDIT_IN, textin, TEXTSIZE+1);
	if (!strlen(textin)) {
		MsgBox(hwnd, MB_OK | MB_ICONINFORMATION, "You haven't entered any codes.");
		return -2;
	}

	// Append newline
	strcat(textin, NEWLINE);

	// Parse text
	s = textin;
	while (*s) {
		// Process line by line
		linelen = IndexOfChr(s, '\r');
		if (!IsEmptySubStr(s, linelen)) {
			s[linelen] = NUL;

			// Get all tokens of the current line
			toknum = 0;
			p = strtok(s, TOK_DELIMITER);
			while (p != NULL) {
				tok[toknum].str = p;
				tok[toknum].type = GetTokenType(p);

				// Allocate more tokens if necessary
				if ((toknum + 1) == tokmax) {
					tokmax += TOK_STEP_NUM;
					tok = (token_t*)realloc(tok, tokmax * sizeof(token_t));
					if (tok == NULL) return -3;
				}

				// Convert TOK_CODE to TOK_CODEADDR + TOK_CODEVAL
				if (tok[toknum].type & TOK_CODE) {
					tok[toknum].str = p;
					tok[toknum].type = TOK_CODEADDR;
					tok[toknum+1].str = p + NUM_DIGITS_OCTET;
					tok[toknum+1].type = TOK_CODEVAL | (tok[toknum].type & TOK_WILDCARD);
					toknum++;
				}
				toknum++;

				// Next token
				p = strtok(NULL, TOK_DELIMITER);
			}

			// Parse line from right to left
			ctrl = 0; // Preceding token type
			for (i = (toknum-1); i >= 0; i--) {
				// A code consists of two hex octets
				if ((tok[i].type & TOK_HEXOCTET) && (ctrl & TOK_HEXOCTET)) {
					tok[i].type = TOK_CODEADDR | (ctrl & TOK_WILDCARD);
					tok[i+1].type = TOK_CODEVAL | (ctrl & TOK_WILDCARD);
				}
				ctrl = tok[i].type;
			}

			// Process tokens, reassemble text
			ctrl = 0; // 1 for newline
			i = 0;
			while (i < toknum) {
#ifdef _DEBUG
				fprintf(fp, "%2i: %s\n", tok[i].type, tok[i].str);
				if (tok[i].type & TOK_CODEADDR)
					fprintf(fp, "%i: %s\n", tok[i+1].type, tok[i+1].str);
#endif
				if (tok[i].type & TOK_CODEADDR) {
					// Convert code address and value
					STR_TO_U32(tok[i++].str, &code[0]);

					if (tok[i].type & TOK_WILDCARD)
						code[1] = 0; // Whatever
					else
						STR_TO_U32(tok[i].str, &code[1]);

					// Decrypt/encrypt code
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

					// Format code address and value for output
					sprintf(codestr, "%08X ", code[0]);

					if (tok[i].type & TOK_WILDCARD) {
						p = tok[i].str;
						StrMakeUpper(p);
						sprintf(&codestr[NUM_DIGITS_OCTET+1], "%s", p);
					} else
						sprintf(&codestr[NUM_DIGITS_OCTET+1], "%08X", code[1]);

					// Append code to output
					if (i > 1) strcat(textout, NEWLINE);
					strcat(textout, codestr);
					ctrl = 1;
				} else {
					// Append string to output
					if (ctrl) {
						strcat(textout, NEWLINE);
						ctrl = 0;
					} else if (i > 0) strcat(textout, " ");
					strcat(textout, tok[i].str);
				}
				i++;
			}
			strcat(textout, NEWLINE);
		}
		// Next line
		s += linelen + 2;
	}

#ifdef _DEBUG
	fclose(fp);

	// Show CRC to check the parser's work
	MsgBox(hwnd, MB_ICONINFORMATION | MB_OK, "CRC16: 0x%04X",
		GetCrc16(textout, strlen(textout)));
#endif
	// Display result
	SetDlgItemText(hwnd, IDM_EDIT_OUT, textout);

	free(tok);

	return 0;
}

/*
 * A MessageBox() replacement with format string support.
 */
int MsgBox(HWND hwnd, UINT uType, const char *szFormat, ...)
{
	char buf[1024];
	va_list ap;

	va_start(ap, szFormat);
	vsprintf(buf, szFormat, ap);
	va_end(ap);

	return MessageBox(hwnd, buf, APP_NAME, uType);
}

/*
 * Initializes the OPENFILENAME structure.
 */
void InitOfn(OPENFILENAME *ofn, HWND hwnd, char *szFileName, char *szTitleName)
{
	static const char szFilter[] = \
		"Text Files (*.txt)\0*.txt\0" \
		"All Files (*.*)\0*.*\0" \
		"\0";

	ofn->lStructSize	= sizeof(OPENFILENAME);
	ofn->hwndOwner		= hwnd;
	ofn->hInstance		= NULL;
	ofn->lpstrFilter	= szFilter;
	ofn->lpstrCustomFilter	= NULL;
	ofn->nMaxCustFilter	= 0;
	ofn->nFilterIndex	= 0;
	ofn->lpstrFile		= szFileName;
	ofn->nMaxFile		= MAX_PATH;
	ofn->lpstrFileTitle	= szTitleName;
	ofn->nMaxFileTitle	= MAX_PATH;
	ofn->lpstrInitialDir	= NULL;
	ofn->lpstrTitle		= NULL;
	ofn->Flags		= 0;
	ofn->nFileOffset	= 0;
	ofn->nFileExtension	= 0;
	ofn->lpstrDefExt	= "txt";
	ofn->lCustData		= 0L;
	ofn->lpfnHook		= NULL;
	ofn->lpTemplateName	= NULL;
}

/*
 * Creates a "Open" dialog box.
 */
BOOL FileOpenDlg(OPENFILENAME *ofn)
{
     ofn->Flags = OFN_FILEMUSTEXIST | OFN_HIDEREADONLY | OFN_NOCHANGEDIR;

     return GetOpenFileName(ofn);
}

/*
 * Creates a "Save" dialog box.
 */
BOOL FileSaveDlg(OPENFILENAME *ofn)
{
     ofn->Flags = OFN_OVERWRITEPROMPT;

     return GetSaveFileName(ofn);
}

/*
 * Loads the contents of a text file into the input dialog box.
 */
BOOL LoadTextFile(HWND hwnd, char *szFileName)
{
	char *buf;
	DWORD dwBytesRead;
	HANDLE hFile;
	int iFileLength, ret;

	// Open file for reading
	hFile = CreateFile(szFileName, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
	if (hFile == INVALID_HANDLE_VALUE) {
		MsgBox(hwnd, MB_ICONERROR | MB_OK, "Can't read from file \"%s\"", szFileName);
		return FALSE;
	}

	// Check file size
	iFileLength = GetFileSize(hFile, NULL);
	if (iFileLength > TEXTSIZE) {
		ret = MsgBox(hwnd, MB_ICONQUESTION | MB_YESNO,
			"%s can only load the first %i bytes of this file (%i bytes total).\n\nOpen anyway?",
			APP_NAME, TEXTSIZE, iFileLength);
		if (ret == IDNO) {
			CloseHandle(hFile);
			return FALSE;
		}
		iFileLength = TEXTSIZE;
	}

	// Allocate memory
	buf = (char*)malloc(iFileLength + 1);
	if (buf == NULL) {
		MsgBox(hwnd, MB_ICONERROR | MB_OK, "Unable to allocate %i bytes.", iFileLength + 1);
		CloseHandle(hFile);
		return FALSE;
	}

	// Read from file into buffer
	ReadFile(hFile, buf, iFileLength, &dwBytesRead, NULL);
	CloseHandle(hFile);

	// Set text in input dialog box
	buf[iFileLength] = '\0';
	SetDlgItemText(hwnd, IDM_EDIT_IN, buf);
	free(buf);

	return TRUE;
}

/*
 * Saves the text in the output dialog box to a text file.
 */
BOOL SaveTextFile(HWND hwnd, char *szFileName)
{
	char *buf;
	DWORD dwBytesWritten;
	HANDLE hFile;
	HWND hCtrl;
	int iLength;

	// Open file for writing
	hFile = CreateFile(szFileName, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, 0, NULL);
	if (hFile == INVALID_HANDLE_VALUE) {
		MsgBox(hwnd, MB_ICONERROR | MB_OK, "Can't write to file \"%s\"", szFileName);
		return FALSE;
	}

	// Get size of output dialog box text
	hCtrl = GetDlgItem(hwnd, IDM_EDIT_OUT);
	iLength = GetWindowTextLength(hCtrl);

	// Allocate memory
	buf = (char*)malloc(iLength + 1);
	if (buf == NULL) {
		MsgBox(hwnd, MB_ICONERROR | MB_OK, "Unable to allocate %i bytes.", iLength + 1);
		CloseHandle(hFile);
		return FALSE;
	}

	// Write text to file
	GetWindowText(hCtrl, buf, iLength + 1);
	WriteFile(hFile, buf, iLength, &dwBytesWritten, NULL);
	CloseHandle(hFile);
	free(buf);

	return TRUE;
}

/*
 * Initializes the application.
 */
int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, PSTR szCmdLine, int iCmdShow)
{
	HACCEL hAccel;
	HWND hwnd;
	MSG msg;
	WNDCLASS wndclass;

	// Register custom dialog box class
	wndclass.style		= CS_HREDRAW | CS_VREDRAW;
	wndclass.lpfnWndProc	= WndProc;
	wndclass.cbClsExtra	= 0;
	wndclass.cbWndExtra	= DLGWINDOWEXTRA;
	wndclass.hInstance	= hInstance;
	wndclass.hIcon		= LoadIcon(hInstance, IDM_ICON);
	wndclass.hCursor	= LoadCursor(NULL, IDC_ARROW);
	wndclass.hbrBackground	= NULL;
	wndclass.lpszMenuName	= IDM_MENU;
	wndclass.lpszClassName	= APP_NAME;
	RegisterClass(&wndclass);

	// Create main dialog
	hwnd = CreateDialog(hInstance, IDD_MAIN, NULL, MainDlgProc);
	if (hwnd == NULL) {
		MsgBox(NULL, MB_ICONERROR | MB_OK, "Could not create main dialog.");
		return 0;
	}
	ShowWindow(hwnd, iCmdShow);

	// Load shortcuts
	hAccel = LoadAccelerators(hInstance, IDM_ACCEL);

	// Message loop
	while (GetMessage(&msg, NULL, 0, 0)) {
		if (!TranslateAccelerator(hwnd, hAccel, &msg)) {
			if (!IsDialogMessage(hwnd, &msg)) {
				TranslateMessage(&msg);
				DispatchMessage(&msg);
			}
		}
	}

	return msg.wParam;
}

/*
 * The window procedure.
 */
LRESULT CALLBACK WndProc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	HMENU hMenu;
	static HINSTANCE hInst;
	static HWND hCtrl;
	static OPENFILENAME ofn;
	static char szFileName[MAX_PATH];
	static char szTitleName[MAX_PATH];
	int iSelBeg, iSelEnd, iEnable;

	switch (message) {
		case WM_CREATE:
			hInst = ((LPCREATESTRUCT)lParam)->hInstance;

			// Change title bar to include version number
			SetWindowText(hwnd, TITLEBAR_TEXT);

			// Add system menu "About"
			hMenu = GetSystemMenu(hwnd, FALSE);
			AppendMenu(hMenu, MF_SEPARATOR, 0, NULL);
			AppendMenu(hMenu, MF_STRING, IDM_SYS_ABOUT, "About...");

			// Init OPENFILENAME
			InitOfn(&ofn, hwnd, szFileName, szTitleName);
			return 0;

/*		case WM_SETFOCUS:
			SetFocus(GetDlgItem(hwnd, IDM_EDIT_IN)); // Doesn't work?
			return 0;
*/
		case WM_INITMENUPOPUP:
			switch (LOWORD(lParam)) {
				case 0: // Submenu "File"
					// Enable "Save as text file" if there is any text to save
					EnableMenuItem((HMENU)wParam, IDM_MENU_SAVE,
						GetWindowTextLength(GetDlgItem(hwnd, IDM_EDIT_OUT)) ? MF_ENABLED : MF_GRAYED);
					break;

				case 1: // Submenu "Edit"
					hCtrl = GetFocus();
					if (hCtrl == GetDlgItem(hwnd, IDM_EDIT_IN)) { // Input box has focus
						// Enable "Undo" if edit-control operation can be undone
						EnableMenuItem((HMENU)wParam, IDM_MENU_UNDO,
							SendMessage(hCtrl, EM_CANUNDO, 0, 0) ? MF_ENABLED : MF_GRAYED);

						// Enable "Paste" if clipboard contains text
						EnableMenuItem((HMENU)wParam, IDM_MENU_PASTE,
							IsClipboardFormatAvailable (CF_TEXT) ? MF_ENABLED : MF_GRAYED);

						// Enable "Cut", "Copy" and "Delete" if some text is selected
						SendMessage(hCtrl, EM_GETSEL, (WPARAM)&iSelBeg, (LPARAM)&iSelEnd);
						iEnable = (iSelBeg != iSelEnd) ? MF_ENABLED : MF_GRAYED;
						EnableMenuItem((HMENU)wParam, IDM_MENU_CUT, iEnable);
						EnableMenuItem((HMENU)wParam, IDM_MENU_COPY, iEnable);
						EnableMenuItem((HMENU)wParam, IDM_MENU_DEL, iEnable);

						// Enable "Select All" and "Clear" if there is any text
						iEnable = GetWindowTextLength(hCtrl) ? MF_ENABLED : MF_GRAYED;
						EnableMenuItem((HMENU)wParam, IDM_MENU_SELECT_ALL, iEnable);
						EnableMenuItem((HMENU)wParam, IDM_MENU_CLEAR, iEnable);

					} else if (hCtrl == GetDlgItem(hwnd, IDM_EDIT_OUT)) { // Output box has focus
						// Enable "Copy" if some text is selected
						SendMessage(hCtrl, EM_GETSEL, (WPARAM)&iSelBeg, (LPARAM)&iSelEnd);
						iEnable = (iSelBeg != iSelEnd) ? MF_ENABLED : MF_GRAYED;
						EnableMenuItem((HMENU)wParam, IDM_MENU_COPY, iEnable);

						// Enable "Select All" and "Clear" if there is any text
						iEnable = GetWindowTextLength(hCtrl) ? MF_ENABLED : MF_GRAYED;
						EnableMenuItem((HMENU)wParam, IDM_MENU_SELECT_ALL, iEnable);
						EnableMenuItem((HMENU)wParam, IDM_MENU_CLEAR, iEnable);

						// Disable the other items
						EnableMenuItem((HMENU)wParam, IDM_MENU_UNDO, MF_GRAYED);
						EnableMenuItem((HMENU)wParam, IDM_MENU_CUT, MF_GRAYED);
						EnableMenuItem((HMENU)wParam, IDM_MENU_PASTE, MF_GRAYED);
						EnableMenuItem((HMENU)wParam, IDM_MENU_DEL, MF_GRAYED);
					} else {
						// Disable all menu items
						EnableMenuItem((HMENU)wParam, IDM_MENU_UNDO, MF_GRAYED);
						EnableMenuItem((HMENU)wParam, IDM_MENU_CUT, MF_GRAYED);
						EnableMenuItem((HMENU)wParam, IDM_MENU_COPY, MF_GRAYED);
						EnableMenuItem((HMENU)wParam, IDM_MENU_PASTE, MF_GRAYED);
						EnableMenuItem((HMENU)wParam, IDM_MENU_DEL, MF_GRAYED);
						EnableMenuItem((HMENU)wParam, IDM_MENU_SELECT_ALL, MF_GRAYED);
						EnableMenuItem((HMENU)wParam, IDM_MENU_CLEAR, MF_GRAYED);
					}
					break;

				case 2: // Submenu "Codes"
					iEnable = GetWindowTextLength(GetDlgItem(hwnd, IDM_EDIT_IN)) ? MF_ENABLED : MF_GRAYED;
					EnableMenuItem((HMENU)wParam, IDM_MENU_DECRYPT, iEnable);
					EnableMenuItem((HMENU)wParam, IDM_MENU_ENCRYPT, iEnable);
					EnableMenuItem((HMENU)wParam, IDM_MENU_REFORMAT, iEnable);
					break;
			}
			return 0;

		case WM_COMMAND:
			// Set focus to input box
			switch (LOWORD(wParam)) {
				case IDM_DECRYPT:
				case IDM_ENCRYPT:
				case IDM_REFORMAT:
				case IDM_CLEAR_IN:
				case IDM_CLEAR_OUT:
				case IDM_MENU_OPEN:
					SetFocus(GetDlgItem(hwnd, IDM_EDIT_IN));
			}

			switch (LOWORD(wParam)) {
				case IDM_DECRYPT:
				case IDM_MENU_DECRYPT:
					ParseText(hwnd, MODE_DECRYPT);
					return 0;

				case IDM_ENCRYPT:
				case IDM_MENU_ENCRYPT:
					ParseText(hwnd, MODE_ENCRYPT);
					return 0;

				case IDM_REFORMAT:
				case IDM_MENU_REFORMAT:
					ParseText(hwnd, MODE_REFORMAT);
					return 0;

				case IDM_CLOSE:
				case IDM_MENU_EXIT:
					PostQuitMessage(0);
					return 0;

				case IDM_CLEAR_IN:
					SetDlgItemText(hwnd, IDM_EDIT_IN, "");
					return 0;

				case IDM_CLEAR_OUT:
					SetDlgItemText(hwnd, IDM_EDIT_OUT, "");
					return 0;

				case IDM_MENU_CLEAR:
					SetDlgItemText(hwnd, GetDlgCtrlID(hCtrl), "");
					return 0;

				case IDM_COMMON_V7:
				case IDM_MENU_COMMON_V7:
					v7common ^= 1;
					CheckMenuItem(GetMenu(hwnd), IDM_MENU_COMMON_V7, v7common ? MF_CHECKED : MF_UNCHECKED);
					if (LOWORD(wParam) == IDM_MENU_COMMON_V7)
						CheckDlgButton(hwnd, IDM_COMMON_V7, v7common ? BST_CHECKED : BST_UNCHECKED);
					return 0;

				case IDM_MENU_OPEN:
					if (FileOpenDlg(&ofn))
						LoadTextFile(hwnd, szFileName);
					return 0;

				case IDM_MENU_SAVE:
					if (FileSaveDlg(&ofn))
						SaveTextFile(hwnd, szFileName);
					return 0;

				case IDM_MENU_UNDO:
					SendMessage(hCtrl, WM_UNDO, 0, 0);
					return 0;

				case IDM_MENU_CUT:
					SendMessage(hCtrl, WM_CUT, 0, 0);
					return 0;

				case IDM_MENU_COPY:
					SendMessage(hCtrl, WM_COPY, 0, 0);
					return 0;

				case IDM_MENU_PASTE:
					SendMessage(hCtrl, WM_PASTE, 0, 0);
					return 0;

				case IDM_MENU_DEL:
					SendMessage(hCtrl, WM_CLEAR, 0, 0);
					return 0;

				case IDM_MENU_SELECT_ALL:
					SendMessage(hCtrl, EM_SETSEL, 0, -1);
					return 0;

				case IDM_MENU_ABOUT:
					DialogBox(hInst, IDD_ABOUTBOX, hwnd, AboutDlgProc);
					return 0;
			}
			break;

		case WM_SYSCOMMAND:
			switch (LOWORD(wParam)) {
				case IDM_SYS_ABOUT:
					DialogBox(hInst, IDD_ABOUTBOX, hwnd, AboutDlgProc);
					return 0;
			}
			break;

		case WM_CLOSE:
			PostQuitMessage(0);
			return 0;
	}

	// Not DefWindowProc()!
	return DefDlgProc(hwnd, message, wParam, lParam);
}

/*
 * Callback function for Main dialog.
 */
BOOL CALLBACK MainDlgProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam)
{
	LOGFONT lf;
	static HFONT hFont;

	switch (message) {
		case WM_INITDIALOG:
			// Setup font
			hFont = (HFONT)SendDlgItemMessage(hDlg, IDM_EDIT_IN, WM_GETFONT, 0, 0);
			GetObject(hFont, sizeof(LOGFONT), &lf);
			strcpy(lf.lfFaceName, "Courier");
			hFont = CreateFontIndirect(&lf);
			SendDlgItemMessage(hDlg, IDM_EDIT_IN, WM_SETFONT, (WPARAM)hFont, FALSE);
			SendDlgItemMessage(hDlg, IDM_EDIT_OUT, WM_SETFONT, (WPARAM)hFont, FALSE);

			// Set text limit
			SendDlgItemMessage(hDlg, IDM_EDIT_IN, EM_SETLIMITTEXT, TEXTSIZE, 0);
			return TRUE;

		case WM_NCDESTROY:
			if (hFont)
				DeleteObject(hFont);
			break;

		// Other messages are processed by the window procedure.
	}

	return FALSE;
}

/*
 * Subclass procedure that sets the hand cursor when moving the cursor over a
 * link (e.g. URL, e-mail address).
 */
WNDPROC g_wndpStatic;

LRESULT CALLBACK LinkProc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	switch (message) {
		case WM_SETCURSOR:
			SetCursor(LoadCursor(NULL, IDC_HAND));
			return TRUE;
	}

	return CallWindowProc(g_wndpStatic, hwnd, message, wParam, lParam);
}

/*
 * Callback function for About box.
 */
BOOL CALLBACK AboutDlgProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam)
{
	LOGFONT lf;
	static HFONT hFont;

	switch (message) {
		case WM_INITDIALOG:
			// Change text to include version number
			SetDlgItemText(hDlg, IDC_APP_VERSION, APP_VERSION);

			// Underline e-mail address
			hFont = (HFONT)SendDlgItemMessage(hDlg, IDC_EMAIL, WM_GETFONT, 0, 0);
			GetObject(hFont, sizeof(lf), &lf);
			lf.lfUnderline = TRUE;
			hFont = CreateFontIndirect(&lf);
			SendDlgItemMessage(hDlg, IDC_EMAIL, WM_SETFONT, (WPARAM)hFont, (LPARAM)TRUE);

			// Set subclass procedure for e-mail address
			g_wndpStatic = (WNDPROC)SetWindowLongPtr(GetDlgItem(hDlg, IDC_EMAIL), GWLP_WNDPROC, (LONG_PTR)LinkProc);
			return TRUE;

		case WM_COMMAND:
			switch (LOWORD(wParam)) {
				case IDC_EMAIL:
					if (HIWORD(wParam) == STN_CLICKED)
						ShellExecute(hDlg, NULL, EMAIL_ADDR, NULL, NULL, SW_SHOW);
					return TRUE;

				case IDOK:
				case IDCANCEL:
					EndDialog(hDlg, 0);
					return TRUE;
			}
			break;

		case WM_CTLCOLORSTATIC:
			// Set text color for e-mail address
			if (GetDlgItem(hDlg, IDC_EMAIL) == (HWND)lParam) {
				SetTextColor((HDC)wParam, GetSysColor(COLOR_HIGHLIGHT));
				SetBkMode((HDC)wParam, TRANSPARENT);
				return (BOOL)GetStockObject(NULL_BRUSH);
			}
			break;

		case WM_NCDESTROY:
			if (hFont)
				DeleteObject(hFont);
			break;

		case WM_CLOSE:
			EndDialog(hDlg, 0);
			return TRUE;
	}

	return FALSE;
}
