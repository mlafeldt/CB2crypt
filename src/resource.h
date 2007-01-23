/*
 * resource.h -- IDs to use with resource file "resource.res"
 *
 * Copyright (C) 2006-2007 misfire
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

#ifndef _RESOURCE_H_
#define _RESOURCE_H_

// Main dialog
#define IDD_MAIN		"MAIN"
#define IDM_ICON		"LOCK"
#define IDM_MENU		"MENU"
#define IDM_ACCEL		"ACCEL"
#define IDM_EDIT_IN		101
#define IDM_EDIT_OUT		102
#define IDM_DECRYPT		201
#define IDM_ENCRYPT		202
#define IDM_REFORMAT		203
#define IDM_CLOSE		204
#define IDM_CLEAR_IN		205
#define IDM_CLEAR_OUT		206
#define IDM_COMMON_V7		301
#define IDM_SYS_ABOUT		401

// Menu
#define IDM_MENU_OPEN		40001	// File
#define IDM_MENU_SAVE		40002
#define IDM_MENU_EXIT		40003
#define IDM_MENU_UNDO		40010	// Edit
#define IDM_MENU_REDO		40011
#define IDM_MENU_CUT		40012
#define IDM_MENU_COPY		40013
#define IDM_MENU_PASTE		40014
#define IDM_MENU_DEL		40015
#define IDM_MENU_SELECT_ALL	40016
#define IDM_MENU_CLEAR		40017
#define IDM_MENU_OUT_TO_IN	40018
#define IDM_MENU_DECRYPT	40019	// Codes
#define IDM_MENU_ENCRYPT	40020
#define IDM_MENU_REFORMAT	40021
#define IDM_MENU_COMMON_V7	40030	// Options
#define IDM_MENU_BLANK_LINE	40031
#define IDM_MENU_ABOUT		40040	// ?

// About box
#define IDD_ABOUTBOX		"ABOUTBOX"
#define IDC_APP_VERSION		501
#define IDC_EMAIL		502

#endif /*_RESOURCE_H_*/
