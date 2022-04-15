#! /bin/bash

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests"

# currently requires nightly
cargo +nightly test

# generate html report
grcov ./target/debug/ -s . -t html --llvm --branch --ignore-not-existing -o ./target/debug/coverage/

# open report
open target/debug/coverage/index.html

# deactivate flags
export RUSTFLAGS=""