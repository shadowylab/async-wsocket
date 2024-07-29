check-fmt:
	cargo fmt --all -- --config format_code_in_doc_comments=true --check

fmt:
	cargo fmt --all -- --config format_code_in_doc_comments=true

check: fmt
	cargo check
	cargo check --features tor
	cargo check --features socks
	cargo check --target wasm32-unknown-unknown
	cargo clippy -- -D warnings
	cargo clippy --features tor -- -D warnings
	cargo clippy --features socks -- -D warnings
	cargo clippy --target wasm32-unknown-unknown -- -D warnings

precommit: fmt check