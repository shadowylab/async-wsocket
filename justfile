#!/usr/bin/env just --justfile

fmt:
    cargo +nightly fmt --all -- --config format_code_in_doc_comments=true

check:
	cargo check
	cargo check --features tor
	cargo check --features socks
	cargo check --target wasm32-unknown-unknown

clippy:
	cargo clippy -- -D warnings
	cargo clippy --features tor -- -D warnings
	cargo clippy --features socks -- -D warnings
	cargo clippy --target wasm32-unknown-unknown -- -D warnings

test:
	cargo test
	cargo test --features tor
	cargo test --features socks

precommit: fmt check clippy test
