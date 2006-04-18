

---======================================---
   CB2crypt - CodeBreaker PS2 Crypto Program
   Copyright (C) 2006 misfire
---=========================================---


Welcome to CB2crypt! This program will decrypt and encrypt _any_ cheat code for
CodeBreaker PS2, including version 7 or higher.


---============---
   Version history
---===============---

v1.11 (Apr 18 2006)
* A small parser upgrade:
  Lines are no longer parsed from left to right, but from right to left.
  This way, such lines are now processed correctly:
     1st place Set at 99999998   8D39AD57 23332B61
  Decrypted output:
     1st place Set at 99999998
     205EC200 05F5E0FE

v1.1 (Apr 13 2006)
* Significantly improved the parser functionality.
  Now you can directly copy and paste the codes from CMGSCCC.com
+ Added a check box to manually enable/disable the commonly used V7 encryption
  (B4336FA9 4DFEFB79). So you no longer have to place this seed code at the top
  of the code list.
* Minor changes to source code and makefile
* Rewrote most parts of this readme.

v1.0 (Mar 23 2006)
* Initial public release

(Mar 8 2006)
* First CB V7 codes decrypted.


---===========---
   About CB2crypt
---==============---

Well, the CB V7 code encryption is by far the most complex scheme I've ever
cracked. Reliable encryption algorithms like ARCFOUR(*) or RSA are being used to
obfuscate the cheat codes in an evil manner -- cf. source code for more details.

However, it's still a proprietary encryption scheme with keys built into
software, and thus has been broken like all its predecessors. Will the
manufacturers of cheat devices ever learn this lesson? I don't think so.

(*) Alleged RC4. The algorithm is still the intellectual property of RSADSI.
I hope Pelican obtained a license for using RC4 in a commercial product...


---=====---
   GUI help
---========---

Let's start with what each button does:

"Clear" (left site) - Clears the input dialog box.

"Clear" (right site) - Clears the output dialog box.

"Decrypt" - Reformats the text and decrypts all codes from the input box. The
result is displayed in the output box.

"Encrypt" - Reformats the text and encrypts all codes from the input box. The
result is displayed in the output box.

"Reformat only" - Only reformats the text from the input box. The result is
displayed in the output box.

"Close" - Exits the program.

Enable the check box "Use common V7 encryption" to set the seed code
B4336FA9 4DFEFB79, see chapter "Beefcode" for more details.

Selecting the system menu "About" will display some general program information.


---=====---
   Beefcode
---========---

By default, CB2crypt processes codes using the old, crappy CB V1 encryption.
In order to switch to the new V7 scheme, there must be a special seed code
-- "Beefcode" -- at the top of the code list.

There are 3 different code types that affect the encryption:

[1] Change encryption seeds

BEEFC0DE vvvvvvvv

v = seed value

[2] Change encryption seeds (extended)

BEEFC0DF vvvvvvvv
wwwwwwww wwwwwwww

v = seed value
w = extra seed value


[1] and [2] affect _all_ following codes in the code list! Also, they go
together very well. You can even use them several times in any order.

For example:
BEEFC0DE 00000000
BEEFC0DF B16B00B5
01234567 89ABCDEF
BEEFC0DE DEADFACE
(normal code)

So far, the only seed code I've seen is B4336FA9 4DFEFB79 which decrypts to
BEEFC0DE 00000000. But this may change. ;)


[3] Change encryption options (NOT SUPPORTED!!)
(Must be used with BEEFC0DE/BEEFC0DF!)

FFFFFFFF 000xnnnn

x = options:
      0 - V7 encryption off, use no/V1 encryption
      1 - V7 encryption on (pretty useless)
      2 - ?
      3 - ?
n = number of code lines

This type changes the encryption options to x for the next n code lines.
If n = 0, then _all_ following codes are affected; this can't be cancelled!

For example, FFFFFFFF 00000000 will allow you to use raw codes after using V7-
encrypted codes, see "Use of unofficial codes along with official codes."

However, CB2crypt does _not_ support this kind of code, as it would only
complicate matters. Looking at the original ASM coding, it appears to be not
even implemented properly in CB.

You might still use the code FFFFFFFF 00000000. With a code value of 0, it seems
to work right, exceptionally.


---===========================---
   Simple code encrypting example
---==============================---

First, without any seed code:
(CB V1 encryption)

2033BA10 FFFFFFFF -> 2ABB1147 FFFFFFFF
2033BA14 FFFFFFFF -> 2AA71147 FFFFFFFF

After putting Beefcode in front of the codes:
(CB V7 encryption)

BEEFC0DE 00000000 -> B4336FA9 4DFEFB79
2033BA10 FFFFFFFF -> AE17E302 4AE8B74D
2033BA14 FFFFFFFF -> E8466F61 F9A3B3A8

It's not hard to notice the vast difference.
Decryption works the other way round.

IMPORTANT:

With V7, always remember to put the seed code at the top of the codes you want
decrypted/encrypted! Alternatively, enable the check box "Use common V7
encryption."


---==============================================---
   Use of unofficial codes along with official codes
---=================================================---

To add user/competitor created codes to existing official V7 codes, you must do
the following:

Let's say you want to add a code to the game "Tiger Woods PGA Tour 2006", e.g.

Always Full Gamebreaker P1 (GS2V3-encrypted)
2857957C FCE87A65

The first thing to do is to convert the code into raw format, i.e., decrypt it.
(Skip this step if the code isn't encrypted.) Using MAXConvert by Pyriel, this
code decrypts to

Always Full Gamebreaker P1 (raw)
204D98B0 3F800000

Then, determine the seed code by decrypting the official CB "Enable Code".
(It's most likely the first code line.)

"Tiger Woods PGA Tour 2006"
Enable Code
B4336FA9 4DFEFB79
410C34B6 663C5B57
1C4D0A53 7FE27DE4
C81AC878 32FCD351

Decrypted:
BEEFC0DE 00000000 ; This is the seed code, see chapter "Beefcode".
000FFFFE 0000015A
000FFFFF 0000015A
903DD9F8 00832021

Now that you know the seed code, perform one of the following two steps:

[1] Encrypt code with given seed code

BEEFC0DE 00000000
204D98B0 3F800000

Encrypted:
B4336FA9 4DFEFB79
13896CD2 EF2FA92B

Add to code list:
Always Full Gamebreaker P1
13896CD2 EF2FA92B

[2] Change encryption options

Alternatively, you can change the encryption options to use raw codes after
using V7-encrypted codes.

BEEFC0DE 00000000
FFFFFFFF 00000000 ; V7 encryption off

Encrypted:
B4336FA9 4DFEFB79
0CF49BDF A4E29C32

Append these codes to the very end of the V7 code list:
V7 encryption off
0CF49BDF A4E29C32
Always Full Gamebreaker P1
204D98B0 3F800000


I think the rest of the program is pretty much self-explanatory.
Let's get hacking! :)

-misfire


---=======---
   Disclaimer
---==========---

THIS PROGRAM IS NOT LICENSED, ENDORSED, NOR SPONSORED BY SONY COMPUTER
ENTERTAINMENT, INC. NOR PELICAN ACCESSORIES, INC.
ALL TRADEMARKS ARE PROPERTY OF THEIR RESPECTIVE OWNERS.

CB2crypt comes with ABSOLUTELY NO WARRANTY. It is covered by the GNU General
Public License. Please see file COPYING for further information.


---===========---
   Special thanks
---==============---

I want to thank Parasyte for the MAXcrypt/GCNcrypt sources.
Alexander Valyalkin for his great BIG_INT library.
Steve Balogh for his Icon Library.
ps2dev.org for PS2SDK and the other open source projects.
Deutsche Bank AG for CrypTool.
O. Forster for the arithmetic interpreter ARIBAS.
Prof. Dr. J. Seelaender for his lecture/thoughts on number theory.
Bruce Schneier for his best seller book "Applied Cryptography".
Greets to Nachbrenner and all the _real_ code hackers out there!

Finally, thanks to CMX for challenging me. ;)


---====---
   Contact
---=======---

misfire [AT] xploderfreax [DOT] de

If you enjoy this software, please consider a PayPal donation to the email
address above. Thank you.


---======================---
   HACKING NEEDS TO BE FREE!
---=========================---

