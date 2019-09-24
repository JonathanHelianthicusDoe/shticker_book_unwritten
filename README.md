# shticker\_book\_unwritten

[![crates.io](https://img.shields.io/crates/v/shticker_book_unwritten)](https://crates.io/crates/shticker_book_unwritten)
[![AGPL v3+](https://img.shields.io/badge/license-GNU%20AGPL%20v3%2B-663366)](./LICENSE)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/JonathanHelianthicusDoe/shticker_book_unwritten)](https://github.com/JonathanHelianthicusDoe/shticker_book_unwritten)

![shticker\_book\_unwritten logo](./img/shticker_book_unwritten_256x256.png)

A minimal [CLI](https://en.wikipedia.org/wiki/Command-line_interface) launcher
for the [Toontown Rewritten](https://www.toontownrewritten.com/)
[MMORPG](https://en.wikipedia.org/wiki/Massively_multiplayer_online_role-playing_game).
Inspired by [Shticker Book
Rewritten](https://github.com/madsciencecoder/Shticker-Book-Rewritten).

Currently **only** built to support GNU/Linux, because I don&rsquo;t know much
about Windows NT nor about macOS. If you know something about either of those
platforms and want to help out, feel very free to submit a PR or to file an
issue with a description of what can be done to support the platform(s).

## Installing

### From pre-compiled binary

You can get pre-compiled binaries from [the releases
page](https://github.com/JonathanHelianthicusDoe/shticker_book_unwritten/releases)
on GitHub.

### From [crates.io](https://crates.io/)

Requires a distribution of [Rust](https://www.rust-lang.org/)/cargo, which you
can get from [rustup](https://rustup.rs/).

```bash
cargo install shticker_book_unwritten
```

If you already have a version installed and want the latest version to replace
it, you can run:

```bash
cargo install -f shticker_book_unwritten
```

### From GitHub git repository

Requires a distribution of [Rust](https://www.rust-lang.org/)/cargo, which you
can get from [rustup](https://rustup.rs/).

```bash
git clone https://github.com/JonathanHelianthicusDoe/shticker_book_unwritten.git
cd shticker_book_unwritten
cargo rustc --release -- -C target-cpu=native # Or just `cargo build --release`
strip ./target/release/shticker_book_unwritten # Optional
./target/release/shticker_book_unwritten --help
```

The executable name is quite lengthy, so you will probably want to alias it (to
`sbu` or something like that).

## Panicking

shticker\_book\_unwritten uses `#![forbid(unsafe_code)]`, so it should be
impossible for actual [undefined
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
a bug**, and you should [file an issue against the GitHub
repository](https://github.com/JonathanHelianthicusDoe/shticker_book_unwritten/issues)
and/or [submit a pull
request](https://github.com/JonathanHelianthicusDoe/shticker_book_unwritten/pulls)
to fix it. Additionally, undefined behavior is (obviously) also a bug, so
similar steps should be taken if you find UB, but that will necessarily require
using external libraries in a different way due to them being broken and/or
fixing those external libraries upstream.

## Legal

This entire work (including this document &amp; all associated source code) is
licensed to anyone under the terms of the [GNU Affero General Public License,
version 3](https://www.gnu.org/licenses/agpl-3.0.en.html) (or any higher
version, at your option). For the relevant legal text, see the
[LICENSE](./LICENSE) file.

[![GNU AGPL v3+](https://www.gnu.org/graphics/agplv3-with-text-162x68.png "GNU AGPL v3+")](https://www.gnu.org/licenses/agpl-3.0.en.html)

This work contains ([Rust](https://www.rust-lang.org/)-ified) code from [bsdiff
4.3](http://www.daemonology.net/bsdiff/), which is licensed under a slightly
modified version of [the FreeBSD
license](https://www.freebsd.org/copyright/freebsd-license.html). For the
relevant legal text, see the [LICENSE.bsdiff4](./LICENSE.bsdiff4) file.

The shticker\_book\_unwritten logo is licensed to anyone under the terms of the
[Creative Commons Attribution-ShareAlike license, version
4.0](https://creativecommons.org/licenses/by-sa/4.0/) (or any higher version,
at your option). For the relevant legal text, see
[https://creativecommons.org/licenses/by-sa/4.0/legalcode](https://creativecommons.org/licenses/by-sa/4.0/legalcode),
or the [img/LICENSE.imgs](img/LICENSE.imgs) file for a plaintext version.

[![CC BY-SA 4.0](https://i.creativecommons.org/l/by-sa/4.0/88x31.png "CC BY-SA 4.0")](https://creativecommons.org/licenses/by-sa/4.0/)
