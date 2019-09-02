# shticker_book_unwritten

A minimal [CLI](https://en.wikipedia.org/wiki/Command-line_interface) launcher
for the [Toontown Rewritten](https://www.toontownrewritten.com/)
[MMORPG](https://en.wikipedia.org/wiki/Massively_multiplayer_online_role-playing_game).
Inspired by [Shticker Book
Rewritten](https://github.com/madsciencecoder/Shticker-Book-Rewritten).

Currently only built to support Linux, because I don&rsquo;t know much about
Windows NT nor about macOS. If you know something about either of those
platforms and want to help out, feel very free to submit a PR or to file an
issue with a description of what can be done to support the platform(s).

## Installing

Requires a distribution of [Rust](https://www.rust-lang.org/)/cargo, which you
can get from [rustup](https://rustup.rs/).

```bash
git clone https://github.com/JonathanHelianthicusDoe/shticker_book_unwritten.git
cd shticker_book_unwritten
cargo rustc --release -- -C target-cpu=native # Or just `cargo build --release`
strip ./target/release/shticker_book_unwritten # Optional
./target/release/shticker_book_unwritten --help
```

## Legal

This entire work (including this document &amp; all associated source code) is
licensed to anyone under the terms of the [GNU Affero General Public License,
version 3](https://www.gnu.org/licenses/agpl-3.0.en.html) (or higher, at your
option).

[![GNU AGPL v3+](https://www.gnu.org/graphics/agplv3-with-text-162x68.png "GNU AGPL v3+")](https://www.gnu.org/licenses/agpl-3.0.en.html)
