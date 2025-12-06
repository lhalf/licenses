set shell := ["bash", "-euc"]

build:
    cargo build --locked --release

check:
    cargo fmt --check --all
    cargo clippy --all-targets -- -Dwarnings

test: build
    cargo test

check-strict:
    cargo clippy --all-targets --all-features -- -D clippy::pedantic -D clippy::nursery