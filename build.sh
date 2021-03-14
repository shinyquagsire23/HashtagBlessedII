#!/bin/bash
cargo xbuild --target=aarch64-unknown-none.json --release
$DEVKITPRO/devkitA64/bin/aarch64-none-elf-objcopy -O binary target/aarch64-unknown-none/release/hashtag_blessed_ii hashtagblessed.bin
./copy-junk.sh
