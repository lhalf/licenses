set shell := ["bash", "-euc"]

pre-commit: fmt check test 

build:
    cargo build --locked --release

fmt:
    cargo fmt --all

check:
    cargo fmt --check --all
    cargo clippy --all-targets -- -Dwarnings

test: build
    cargo test

check-strict:
    cargo clippy --all-targets --all-features -- -D clippy::pedantic -D clippy::nursery

update: update-readme
    cargo upgrade --incompatible

update-readme: build
    mdsh
