# shticker_book_unwritten

[![crates.io](https://img.shields.io/crates/v/shticker_book_unwritten)](https://crates.io/crates/shticker_book_unwritten)
[![AGPL v3+](https://img.shields.io/badge/license-GNU%20AGPL%20v3.0%2B-663366)](./LICENSE)
[![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/JonathanHelianthicusDoe/shticker_book_unwritten)](https://github.com/JonathanHelianthicusDoe/shticker_book_unwritten)

![shticker_book_unwritten logo](./img/shticker_book_unwritten_256x256.png)

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

## Legal

This entire work (including this document &amp; all associated source code) is
licensed to anyone under the terms of the [GNU Affero General Public License,
version 3](https://www.gnu.org/licenses/agpl-3.0.en.html) (or higher, at your
option). For the relevant legal text, see the [LICENSE](./LICENSE) file.

[![GNU AGPL v3+](https://www.gnu.org/graphics/agplv3-with-text-162x68.png "GNU AGPL v3+")](https://www.gnu.org/licenses/agpl-3.0.en.html)

This work contains ([Rust](https://www.rust-lang.org/)-ified) code from [bsdiff
4.3](http://www.daemonology.net/bsdiff/), which is licensed under a slightly
modified version of [the FreeBSD
license](https://www.freebsd.org/copyright/freebsd-license.html). For the
relevant legal text, see the [LICENSE.bsdiff4](./LICENSE.bsdiff4) file.

The shticker_book_unwritten logo is licensed to anyone under the terms of the
[Creative Commons Attribution-ShareAlike 4.0 International
license](https://creativecommons.org/licenses/by-sa/4.0/). For the relevant
legal text, see
[https://creativecommons.org/licenses/by-sa/4.0/legalcode](https://creativecommons.org/licenses/by-sa/4.0/legalcode),
or the [img/LICENSE.imgs](img/LICENSE.imgs) file for a plaintext version.

[![CC BY-SA 4.0](https://i.creativecommons.org/l/by-sa/4.0/88x31.png "CC BY-SA 4.0")](https://creativecommons.org/licenses/by-sa/4.0/)
