@default:
    just -l 


@check:
    cargo clippy
    
@run:
    cargo run data

@build: check
    cargo build 

@build-fast: check
    cargo build --release