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

@test:
    cargo test

@doc: clean check
    cargo doc --no-deps --open