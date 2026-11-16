# CB2crypt

App to decrypt & encrypt any cheat code for CodeBreaker PS2.

Paste or load a code list on the left, choose Decrypt or Encrypt, and the result appears on the right.
Game titles and code names pass through untouched.

<img src="assets/screenshot.png" width="720" alt="CB2crypt with the encrypted Resident Evil 4 codes on the left and the decrypted codes on the right">

CB2crypt 2.0 is a Rust rebuild of the Windows program from 2006, the first to decrypt CodeBreaker V7 codes.
To learn how the codes work (what a Beefcode is, how V7 differs from V1, and how to add your own codes to an official list), see the original's [readme](https://github.com/mlafeldt/CB2crypt/blob/windows/readme.txt).

## Build and run

With a recent [Rust toolchain](https://rustup.rs):

```console
cargo run --release
```

For a dark theme:

```console
cargo run --release -- --dark
```

CB2crypt is developed and tested on macOS.
The toolkit it is built on also runs on Windows and Linux, but the app is untried there.

## History

I released CB2crypt in March 2006, two weeks after cracking the CodeBreaker V7 encryption, and updated it until v1.4 in January 2007.

Twenty years later I rebuilt it in Rust as version 2 with Claude Code and Fable 5.2.
Fable also tested the app through computer use, clicking through the real window and taking the screenshot above.
It works as the original did, with the same menus, text boxes and buttons, now drawn by the pixel-perfect Windows 3.1 controls of [saudade](https://github.com/roblillack/saudade).
The crypto comes from the [codebreaker](https://github.com/mlafeldt/codebreaker-rs) crate, which grew out of the original's C code.

The original Windows program lives on in the [`windows`](https://github.com/mlafeldt/CB2crypt/tree/windows) branch, and every release from v1.0 to v1.4 keeps its tag.

## License

Copyright (c) Mathias Lafeldt

Licensed under the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT license](LICENSE-MIT), at your option.

The Windows program, up to v1.4, is licensed under the GNU General Public License, version 2 or later.
