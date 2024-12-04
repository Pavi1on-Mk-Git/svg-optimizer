set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

build:
    cargo build

run +ARGS:
    cargo run -- {{ARGS}}

test:
    cargo test

lint:
    cargo fmt
    cargo clippy

docs:
    cargo doc --no-deps --lib --open
