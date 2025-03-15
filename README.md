# shticker\_book\_unwritten

[![crates.io](https://img.shields.io/crates/v/shticker_book_unwritten)](https://crates.io/crates/shticker_book_unwritten)
[![GPL v3+](https://img.shields.io/badge/license-GNU%20GPL%20v3%2B-bd0000)](./LICENSE)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/JonathanHelianthicusDoe/shticker_book_unwritten)](https://github.com/JonathanHelianthicusDoe/shticker_book_unwritten)

![shticker\_book\_unwritten logo](./img/shticker_book_unwritten_256x256.png)

A minimal [CLI](https://en.wikipedia.org/wiki/Command-line_interface) launcher
for the [Toontown Rewritten](https://www.toontownrewritten.com/)
[MMORPG][mmorpg]. Inspired by [Shticker Book
Rewritten](https://github.com/madsciencecoder/Shticker-Book-Rewritten).

Currently builds and functions on GNU/Linux, Windows NT (using the MSVC
toolchain), and macOS (be sure to allow Terminal the ability to monitor
inputs).

**Mirror:**
<https://codeberg.org/JonathanHelianthicusDoe/shticker_book_unwritten>

## Installing

### From pre-compiled binary

You can get pre-compiled binaries from [the releases page][releases] on GitHub.

### From [crates\.io](https://crates.io/)

Requires a distribution of [Rust](https://www.rust-lang.org/)/cargo, which you
can get from [rustup](https://rustup.rs/).

`cargo install` works as normal. Linux users may opt into the use of a [Secret
Service](https://specifications.freedesktop.org/secret-service-spec/latest/)
keyring (e\.g. [KWallet](https://en.wikipedia.org/wiki/KWallet), [GNOME
Keyring](https://en.wikipedia.org/wiki/GNOME_Keyring), etc.) to save account
details by enabling the relevant feature flag with `-Fsecret-store`. If
you&rsquo;ve already a version installed, and want to replace it with the
latest version, use `-f`.

Typically:

```bash
cargo install -Fsecret-store -f shticker_book_unwritten
```

### From GitHub git repository

Requires a distribution of [Rust](https://www.rust-lang.org/)/cargo, which you
can get from [rustup](https://rustup.rs/).

```bash
git clone https://github.com/JonathanHelianthicusDoe/shticker_book_unwritten.git
cd shticker_book_unwritten
cargo rustc -Fsecret-store --release -- -C target-cpu=native # Or just `cargo build -Fsecret-store --release`
./target/release/shticker_book_unwritten --help
```

The executable name is quite lengthy, so you will probably want to alias it (to
`sbu`, or something like that).

## Password management

When not on Linux, and/or when shticker\_book\_unwritten is built with the
default features, stored passwords are stored **in plain text** on your
filesystem. To avoid this security hazard, you may&hellip;:

- &hellip;Compile with `-Fsecret-store`, if on Linux. In this case,
  shticker\_book\_unwritten will use your [Secret
  Service](https://specifications.freedesktop.org/secret-service-spec/latest/)
  keyring (e\.g. [KWallet](https://en.wikipedia.org/wiki/KWallet), [GNOME
  Keyring](https://en.wikipedia.org/wiki/GNOME_Keyring), etc.).
- &hellip;Not use shticker\_book\_unwritten to store passwords, and instead use
  a separate [password manager](https://en.wikipedia.org/wiki/Password_manager)
  app.

By default, when not provided with a config file, shticker\_book\_unwritten
will ask you whether you want your passwords to be saved. Nonetheless, if
you&rsquo;re uncertain, and you want to ensure that shticker\_book\_unwritten
is not managing any of your passwords, then you may:

1. Use shticker\_book\_unwritten&rsquo;s command mode to run
   `accounts savepws false`.
2. Use `accounts` to list all saved accounts.
3. Use `accounts forget` for each individual account that has an associated
   password.

## Panicking

shticker\_book\_unwritten uses `#![forbid(unsafe_code)]`, so it should (barring
compiler bugs) be impossible for actual [undefined
behavior](https://en.wikipedia.org/wiki/Undefined_behavior) to occur unless
some dependency of shticker\_book\_unwritten is written using `unsafe` code in
an [unsound](https://en.wikipedia.org/wiki/Soundness) way. However, although
shticker\_book\_unwritten is written intentionally to avoid
[panicking](https://doc.rust-lang.org/std/macro.panic.html) for any reason
whatsoever, it is not (in general) possible in Rust to [statically
guarantee](https://en.wikipedia.org/wiki/Rice%27s_theorem) that a program is
[panic-free for all possible
inputs](https://en.wikipedia.org/wiki/Partial_function#Total_function).

If you find a way to make shticker\_book\_unwritten panic, **that is considered
a bug**, and you should [file an issue against the GitHub repository][issues]
and/or [submit a pull request][pulls] to fix it. Additionally, undefined
behavior is (obviously) also a bug, so similar steps should be taken if you
find UB, but that will necessarily require using external libraries in a
different way due to them being broken and/or fixing those external libraries
upstream.

## Legal

This entire work (including this document &amp; all associated source code) is
licensed to anyone under the terms of the [GNU General Public License, version
3](https://www.gnu.org/licenses/gpl-3.0.html) (or any later version of the same
license, at the licensee&rsquo;s option). For the relevant legal text, see the
[LICENSE](./LICENSE) file.

[![GNU GPL v3+](https://www.gnu.org/graphics/gplv3-or-later.png
"GNU GPL v3+")](https://www.gnu.org/licenses/gpl-3.0.html)

This work contains ([Rust](https://www.rust-lang.org/)-ified) code from [bsdiff
4.3](http://www.daemonology.net/bsdiff/), which is licensed under a slightly
modified version of [the FreeBSD
license](https://www.freebsd.org/copyright/freebsd-license.html). For the
relevant legal text, see the [LICENSE.bsdiff4](./LICENSE.bsdiff4) file.

The shticker\_book\_unwritten logo is licensed to anyone under the terms of the
[Creative Commons Attribution-ShareAlike license, version
4.0](https://creativecommons.org/licenses/by-sa/4.0/) (or any later version of
the same license, at the licensee&rsquo;s option). For the relevant legal text,
see [https://creativecommons.org/licenses/by-sa/4.0/legalcode][cc-by-sa], or
the [img/LICENSE.imgs](img/LICENSE.imgs) file for a plaintext version.

[![CC BY-SA 4.0+](https://i.creativecommons.org/l/by-sa/4.0/88x31.png
"CC BY-SA 4.0+")](https://creativecommons.org/licenses/by-sa/4.0/)

<small>Versions of shticker\_book\_unwritten prior to 1.0.0 were licensed under
the terms of the [GNU Affero General Public License, version
3](https://www.gnu.org/licenses/agpl-3.0.html) or later.</small>

[mmorpg]: https://en.wikipedia.org/wiki/Massively_multiplayer_online_role-playing_game
[releases]: https://github.com/JonathanHelianthicusDoe/shticker_book_unwritten/releases
[issues]: https://github.com/JonathanHelianthicusDoe/shticker_book_unwritten/issues
[pulls]: https://github.com/JonathanHelianthicusDoe/shticker_book_unwritten/pulls
[cc-by-sa]: https://creativecommons.org/licenses/by-sa/4.0/legalcode
[cargo-deny]: https://github.com/EmbarkStudios/cargo-deny
