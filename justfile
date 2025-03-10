#!/usr/bin/env just --justfile

@default:
    just -f {{ justfile() }} --list --no-aliases

clippy:
    cargo clippy --all-targets --all-features --workspace -- -D warnings

fmt:
    cargo fmt --all
    just -f {{ justfile() }} --unstable --fmt

fmt-check:
    cargo fmt --all -- --check
    just -f {{ justfile() }} --unstable --fmt --check

docs $RUSTDOCFLAGS="-D warnings":
    cargo doc --no-deps --document-private-items --all-features --workspace

test:
    cargo test --all-features --workspace

check-and-test: clippy docs fmt-check test
