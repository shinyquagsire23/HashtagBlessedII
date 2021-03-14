# HashtagBlessedII

Nintendo Switch/T210 hypervisor, being rewritten in Rust

* See LICENSE.md for license deets

NOTE: Currently only adapted for 8.0.0

## Building
* Requires `0_kernel_80060000.bin` from pk2
* Install `rustup`
* `rustup update`
* `rustup default nightly`
* `cargo install cargo-xbuild`
* `cargo install cargo-binutils`
* `rustup component add llvm-tools-preview`
* `rustup component add rust-src`
* `build.sh`

## Running
* HTB2 replaces the kernel in its entirety, PK2/EL3 dropdown entrypoint should be set to 0xD0000000. HTB2 will extract the kernel from its data, patch it and execute it.
