#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
set -euo pipefail
cd "$SRC/project"
rustc +nightly --version
cargo +nightly --version
cargo +nightly fuzz --version
# Explicit nightly overrides the production rust-toolchain.toml. The OSS-Fuzz
# image supplies its matching cargo-fuzz and sanitizer toolchain.
# cargo-fuzz 0.13 has no --locked option. Fetch the exact lock, forbid network
# resolution during compilation and fail if cargo-fuzz rewrites the lock.
locked_copy=$(mktemp)
trap 'rm -f "$locked_copy"' EXIT
cp fuzz/Cargo.lock "$locked_copy"
cargo +nightly fetch --locked --manifest-path fuzz/Cargo.toml
CARGO_NET_OFFLINE=true cargo +nightly fuzz build --target x86_64-unknown-linux-gnu --release --debug-assertions
cmp "$locked_copy" fuzz/Cargo.lock
for target in kyaml json jcs dhall; do
    install -m 755 "fuzz/target/x86_64-unknown-linux-gnu/release/$target" "$OUT/$target"
    cp "fuzz/options/$target.options" "$OUT/$target.options"
    (cd "fuzz/corpus/$target" && zip -q -j "$OUT/${target}_seed_corpus.zip" ./*)
done
