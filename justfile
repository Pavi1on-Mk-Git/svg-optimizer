set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

build:
    cargo build

run +ARGS:
    cargo run -- {{ARGS}}

test:
    cargo test

docs:
    cargo doc --no-deps --lib --open
