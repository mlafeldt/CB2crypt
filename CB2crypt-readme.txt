

---======================================---
   CB2crypt - CodeBreaker PS2 Crypto Program
   Copyright (C) 2006 misfire
---=========================================---


Welcome to CB2crypt! This program will decrypt and encrypt _any_ cheat code for
CodeBreaker PS2, including version 7 or higher.


---============---
   Version history
---===============---

v1.0 (Mar 23 2006)
* Initial public release

(Mar 8 2006)
* First CB V7 codes decrypted.


---===========---
   About CB2crypt
---==============---

Well, the CB V7 code encryption is by far the most complex scheme I've ever
cracked. It's also the first one where reliable encryption algorithms like
ARCFOUR(*) or RSA are being used -- cf. source code for more details.

However, it's still a proprietary encryption scheme with keys built into
software, and thus has been broken like all its predecessors. Will the
manufacturers of cheat devices ever learn this lesson? I don't think so.

(*) Alleged RC4. The algorithm is still the intellectual property of RSADSI.
I hope Pelican obtained a license for using RC4 in a commercial product...


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


[1] and [2] affect all following codes in the code list! Also, they go together
very well. You can even use them several times in any order. For example:

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

For example, FFFFFFFF 00000000 will allow you to use raw codes after using V7
encrypted codes.

However, CB2crypt does _not_ support this kind of code, as it would only
complicate matters. Looking at the original ASM coding, it appears to be not
even implemented properly in CB. Actually, it's just listed for the sake of
completeness, and I don't recommend using it at all.


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


---======================---
   HACKING NEEDS TO BE FREE!
---=========================---

