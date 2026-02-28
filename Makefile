fmt:
	cargo +nightly fmt -- --unstable-features --config imports_granularity=One --config group_imports=One

fmt-check:
	cargo +nightly fmt --check -- --unstable-features --config imports_granularity=One --config group_imports=One

lint:
	cargo clippy --all-targets --all-features -- -D warnings

check:
	cargo check
