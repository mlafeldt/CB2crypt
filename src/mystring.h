/*
 * mystring.h -- Small library of string functions
 *
 * Copyright (C) 2004-2005 misfire
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

#ifndef _MYSTRING_H_
#define _MYSTRING_H_

/* Includes */

#include <stdlib.h>
#include <string.h>
#include "mytypes.h"

/* Defines */

#define NUL	0x00

// White-Space Characters
#define TAB	0x09	// Horizontal tab
#define LF	0x0A	// Line feed (New line)
#define CR	0x0D	// Carriage return
#define SPACE	0x20

/* Declarations */

char *AddStr(const char *s1, const char *s2);
int IndexOfChr(const char *s, char c);
int LastIndexOfChr(const char *s, char c);
int MakePrintStr(char *s, char c);
void RemCmtFromStr(char *s);
int SetMaxStrLen(char *s, int max);
int TrimStr(char *s);
int _isascii(int c);
int IsCmtStr(const char *s);
int IsEmptyStr(const char *s);
int IsEmptySubStr(const char *s, int count);
int IsPrintStr(const char *s);

#endif /*_MYSTRING_H_*/
