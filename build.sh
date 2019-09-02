#!/usr/bin/env bash

set -ex

cargo update --aggressive
if [[ "$1" =~ ^-?d(ebug)?$ ]]; then
    cargo build
else
    cargo rustc --release -- -C target-cpu=native
    strip ./target/release/shticker_book_unwritten
fi
