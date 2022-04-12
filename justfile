@default:
    just -l 


@check:
    cargo clippy
    
@run:
    # remove old files
    find data -type f -name "*.jsonl" -or -name "*.json" -not -name "sample.json" | xargs rm
    cargo run -- data

@build: check
    cargo build 

@build-fast: check
    cargo build --release