/*
 * mystring.c -- Small library of string functions (stripped)
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

/*
 * Returns the index within s of the first occurrence of the specified char c.
 * If no such char occurs in s, then (-1) is returned.
 */
int IndexOfChr(const char *s, char c)
{
	int i = 0;

	while (s[i] && (s[i] != c)) i++;

	if (s[i] == c) return i;

	return -1;
}

/*
 * Converts s to an uppercase string.
 */
char *StrMakeUpper(char *s)
{
	int i;

	for (i = 0; i < strlen(s); i++)
		s[i] = toupper(s[i]);

	return s;
}

/*
 * Returns TRUE if the first count chars of s are not printable
 * (apart from white space).
 */
int IsEmptySubStr(const char *s, int count)
{
	while (count--) {
		if (isgraph(*s++)) return FALSE;
	}

	return TRUE;
}

/*
 * Returns TRUE if all chars of s are hexadecimal.
 */
int IsHexStr(const char *s)
{
	while (*s) {
		if (!isxdigit(*s++)) return FALSE;
	}

	return TRUE;
}
