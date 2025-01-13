set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

build:
    cargo build

build-release:
    cargo build --release

run *ARGS:
    cargo run -- {{ARGS}}

run-release *ARGS:
    cargo run --release -- {{ARGS}}

test:
    cargo test

lint:
    cargo fmt
    cargo clippy

docs:
    cargo doc --no-deps --open
