/*
 * mystring.c -- Small library of string functions
 *
 * Copyright (C) 2004-2006 misfire
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

#include "mystring.h"

/*	char *AddStr(const char *s1, const char *s2);
 *
 *	AddStr() appends s2 to s1 and returns a pointer to the new string.
 */
char *AddStr(const char *s1, const char *s2)
{
	char *s = (char*)malloc(strlen(s1) + strlen(s2) + 1);

	if (s != NULL) {
		strcpy(s, s1);
		strcat(s, s2);
	}

	return s;
}

/*	int IndexOfChr(const char *s, char c);
 *
 *	IndexOfChr() returns the index within s of the first occurrence of the
 *	specified char c. If no such char occurs in s, then (-1) is returned.
 */
int IndexOfChr(const char *s, char c)
{
	int i = 0;

	while (s[i] && (s[i] != c)) i++;

	if (s[i] == c) return i;

	return -1;
}

/*	int LastIndexOfChr(const char *s, char c);
 *
 *	LastIndexOfChr() returns the index within s of the last occurrence of
 *	the specified char c. If the char does not occur in s, then (-1) is
 *	returned.
 */
int LastIndexOfChr(const char *s, char c)
{
	int i = strlen(s);

	while (i && s[--i] != c);

	if (s[i] == c) return i;

	return -1;
}

/*	int MakePrintStr(char *s, char c);
 *
 *	MakePrintStr() replaces non-printable chars in s with printable char c.
 */
int MakePrintStr(char *s, char c)
{
	int i = 0;

	while (*s) {
		if (!isprint(*s)) {
			*s = c;
			i++;
		}
		s++;
	}

	return i;
}

/*	void RemCmtFromStr(char *s);
 *
 *	RemCmtFromStr() removes a trailing comment (if present) from string s.
 */
void RemCmtFromStr(char *s)
{
	// NUL-terminate string where comment begins
	while (*s) {
		if (IsCmtStr(s)) {
			*s = NUL;
			break;
		}
		s++;
	}
}

/*	int SetMaxStrLen(char *s, int max);
 *
 *	SetMaxStrLen() sets the maximum length of the string s.
 *	If s is longer, it will be shortened to max chars.
 */
int SetMaxStrLen(char *s, int max)
{
	if ((int)strlen(s) <= max) return FALSE;

	s[max] = NUL;

	return TRUE;
}

/*	int TrimStr(char *s);
 *
 *	TrimStr() removes white space from both ends of the string s.
 */
int TrimStr(char *s)
{
	int first = 0;
	int last;
	int slen;
	char *t = s;

	// Return if string is empty
	if (IsEmptyStr(s)) return -1;

	// Get first non-space char
	while (isspace(*t++)) first++;

	// Get last non-space char
	last = strlen(s) - 1;
	t = &s[last];
	while (isspace(*t--)) last--;

	// Kill leading/trailing spaces
	slen = last - first + 1;
	memmove(s, s + first, slen);
	s[slen] = NUL;

	return slen;
}

/*	int _isascii(int c);
 *
 *	_isascii() returns TRUE if c is an ASCII character
 *	(in the range 0x00 - 0x7F).
 */
int _isascii(int c)
{
	return (c >= 0 && c <= 127);
}

/*	int IsCmtStr(const char *s);
 *
 *	IsCmtStr() returns TRUE if s indicates a comment.
 */
int IsCmtStr(const char *s)
{
	return ((strlen(s) >= 2 && !strncmp(s, "//", 2)) ||
		(*s == ';') ||
		(*s == '#')
	);
}

/*	int IsEmptyStr(const char *s);
 *
 *	IsEmptyStr() returns TRUE if s contains no printable chars other than
 *	white space.
 */
int IsEmptyStr(const char *s)
{
	int slen = strlen(s);

	while (slen--) {
		if (isgraph(*s++)) return FALSE;
	}

	return TRUE;
}

/*	int IsEmptySubStr(const char *s, int count);
 *
 *	IsEmptySubStr() returns TRUE if the first count chars of s are not
 *	printable (apart from white space).
 */
int IsEmptySubStr(const char *s, int count)
{
	while (count--) {
		if (isgraph(*s++)) return FALSE;
	}

	return TRUE;
}

/*	int IsHexStr(const char *s);
 *
 *	IsHexStr() returns TRUE if all chars of s are hexadecimal.
 */
int IsHexStr(const char *s)
{
	while (*s) {
		if (!isxdigit(*s++)) return FALSE;
	}

	return TRUE;
}

/*	int IsPrintStr(const char *s);
 *
 *	IsPrintStr() returns TRUE if all chars of s are printable.
 */
int IsPrintStr(const char *s)
{
	while (*s) {
		if (!isprint(*s++)) return FALSE;
	}

	return TRUE;
}
