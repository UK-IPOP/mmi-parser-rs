@default:
    just -l 

@clean:
    cargo clean

@check:
    cargo clippy -- -D warnings --no-deps
    
@run:
    # remove old files
    find data -type f -name "*.jsonl" -or -name "*.json" -not -name "sample.json" | xargs rm
    cargo run -- data

@build: check
    cargo build 

@build-prod: clean check test
    cargo build --release

@test: clean
    cargo test

@test-cov: check
    set CARGO_INCREMENTAL 0
    set RUSTFLAGS "-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off"

    # currently requires nightly
    cargo +nightly test

    # generate html report
    grcov ./target/debug/ -s . -t html --llvm --branch --ignore-not-existing -o ./target/debug/coverage/

    # open report
    open target/debug/coverage/index.html

    # deactivate flags
    set RUSTFLAGS ""

@doc: clean check
    cargo doc --no-deps --open