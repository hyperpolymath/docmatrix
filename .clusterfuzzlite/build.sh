#!/bin/bash -eu

cd $SRC/project
cargo +nightly fuzz build --release
cp fuzz/target/*/release/fuzz_* $OUT/
